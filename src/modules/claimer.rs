use std::{str::FromStr, time::Duration};

use borsh::BorshDeserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, native_token::lamports_to_sol,
    pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    getgrass::{
        api::get_receipt,
        schemas::{ClaimProofEntry, GrassApiResponse, Receipt},
        typedefs::Cluster,
    },
    onchain::{
        constants::{CLAIM_PROGRAM_ID, GRASS_PUBKEY, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
        derive::{derive_ata, derive_claim_status, derive_merkle_distributor},
        ixs::Instructions,
        state::ClaimStatus,
        tx::send_and_confirm_tx,
        typedefs::{ClaimArgs, CreateAtaArgs},
    },
    utils::misc::{pretty_sleep, swap_ip_address},
};

pub async fn claim_grass(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) = db.get_random_account_with_filter(|a| !a.get_claimed()) {
        if let Err(e) = process_account(&provider, account, config).await {
            tracing::error!("{}", e);
        } else {
            account.set_claimed(true);
            db.update();
        };

        pretty_sleep(config.claim_sleep_range).await;
    }

    Ok(())
}

fn prepare_proof(claim_proof_json: &str) -> Vec<[u8; 32]> {
    if let Ok(claim_proof_array) = serde_json::from_str::<Vec<ClaimProofEntry>>(claim_proof_json) {
        claim_proof_array
            .into_iter()
            .filter_map(|entry| {
                if entry.data.type_ != "Buffer" {
                    return None;
                }

                let data_bytes = entry.data.data;

                if data_bytes.len() != 32 {
                    return None;
                }

                let mut buffer = [0u8; 32];
                buffer.copy_from_slice(&data_bytes);
                Some(buffer)
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn extract_version_and_proof(
    receipt: &GrassApiResponse<Receipt>,
) -> eyre::Result<(u32, Vec<[u8; 32]>, u64)> {
    let result = receipt
        .result
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Receipt result is missing"))?;
    let data = result
        .data
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Data is missing in the receipt result"))?;
    let version_number = data
        .version_number
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Version number is missing in the receipt data"))?;
    let claim_proof = data
        .claim_proof
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Claim proof is missing in the receipt data"))?;
    let proof = prepare_proof(claim_proof);

    let allocation = data
        .allocation
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Allocation is missing in the receipt data"))?;
    Ok((*version_number, proof, *allocation))
}

#[allow(clippy::too_many_arguments)]
async fn get_ixs(
    provider: &RpcClient,
    version_number: u32,
    proof: Vec<[u8; 32]>,
    allocation: u64,
    wallet_pubkey: &Pubkey,
    cex_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
    config: &Config,
) -> eyre::Result<Option<Vec<Instruction>>> {
    let (merkle_distributor_pubkey, _) = derive_merkle_distributor(version_number);

    let (claim_status_pubkey, _) = derive_claim_status(wallet_pubkey, &merkle_distributor_pubkey);

    if let Ok(claim_status_data) = provider.get_account_data(&claim_status_pubkey).await {
        let claim_status = ClaimStatus::deserialize(&mut &claim_status_data[8..])?;

        if claim_status.allocation == claim_status.sent_allocation {
            tracing::info!("Already claimed");
            return Ok(None);
        }
    }

    let (token_vault, _) = derive_ata(&merkle_distributor_pubkey, &GRASS_PUBKEY, &TOKEN_PROGRAM_ID);

    let mut ixs = vec![];

    let (wallet_token_ata, _) = derive_ata(wallet_pubkey, &GRASS_PUBKEY, &TOKEN_PROGRAM_ID);
    let token_ata_exist = provider.get_account_data(&wallet_token_ata).await.is_ok();

    if !token_ata_exist {
        let create_ata_args = CreateAtaArgs {
            funding_address: *payer_pubkey,
            associated_account_address: wallet_token_ata,
            wallet_address: *wallet_pubkey,
            token_mint_address: GRASS_PUBKEY,
            token_program_id: TOKEN_PROGRAM_ID,
            instruction: 0,
        };

        ixs.push(Instructions::create_ata(create_ata_args));
    }

    let rent = provider.get_minimum_balance_for_rent_exemption(64).await?;

    let wallet_balance = provider.get_balance(wallet_pubkey).await?;

    if wallet_balance < rent {
        if payer_pubkey == wallet_pubkey {
            tracing::warn!(
                "Wallet doesn't have enough SOL to create ClaimStatus PDA: {} | {}",
                lamports_to_sol(wallet_balance),
                lamports_to_sol(rent)
            );

            return Ok(None);
        } else {
            let transfer_ix =
                solana_sdk::system_instruction::transfer(payer_pubkey, wallet_pubkey, rent);

            ixs.push(transfer_ix);
        }
    }

    let claim_args = ClaimArgs {
        program_id: CLAIM_PROGRAM_ID,
        distributor: merkle_distributor_pubkey,
        mint_token: GRASS_PUBKEY,
        claim_status: claim_status_pubkey,
        from: token_vault,
        to: wallet_token_ata,
        claimant: *wallet_pubkey,
        token_program: TOKEN_PROGRAM_ID,
        system_program: SYSTEM_PROGRAM_ID,
        allocation,
        proof,
    };

    let claim_ix = Instructions::claim(claim_args);

    ixs.push(claim_ix);

    if config.withdraw_to_cex {
        let (cex_token_ata, _) = derive_ata(cex_pubkey, &GRASS_PUBKEY, &TOKEN_PROGRAM_ID);
        let cex_token_ata_exist = provider.get_account_data(&cex_token_ata).await.is_ok();

        if !cex_token_ata_exist {
            let create_ata_args = CreateAtaArgs {
                funding_address: *payer_pubkey,
                associated_account_address: cex_token_ata,
                wallet_address: *cex_pubkey,
                token_mint_address: GRASS_PUBKEY,
                token_program_id: TOKEN_PROGRAM_ID,
                instruction: 0,
            };

            ixs.push(Instructions::create_ata(create_ata_args));
        }

        let transfer_ix = spl_token::instruction::transfer_checked(
            &TOKEN_PROGRAM_ID,
            &wallet_token_ata,
            &GRASS_PUBKEY,
            &cex_token_ata,
            wallet_pubkey,
            &[wallet_pubkey],
            allocation,
            9u8,
        )?;

        ixs.push(transfer_ix);
    }

    Ok(Some(ixs))
}

async fn process_account(
    provider: &RpcClient,
    account: &mut Account,
    config: &Config,
) -> eyre::Result<()> {
    let wallet = account.keypair();
    let wallet_pubkey = account.get_pubkey();
    let cex_pubkey = Pubkey::from_str(account.get_cex_address()).expect("Invalid CEX address");
    let proxy = account.proxy();

    tracing::info!("Wallet address: `{}`", wallet.pubkey());

    if config.mobile_proxies {
        tracing::info!("Changing IP address");
        swap_ip_address(&config.swap_ip_link).await?;
    }

    let receipt = get_receipt(&wallet_pubkey.to_string(), Cluster::Mainnet, proxy.as_ref()).await?;
    let (version_number, proof, allocation) = match extract_version_and_proof(&receipt) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("{}", e);
            account.set_claimed(true);
            return Ok(());
        }
    };

    let alloc = (allocation as f64) / 10f64.powi(9);

    account.set_allocation(alloc);
    tracing::info!("Amount to claim: {} GRASS", alloc);

    let payer_kp = match config.use_external_fee_pay {
        true => Keypair::from_base58_string(&config.external_fee_payer_pk),
        false => wallet.insecure_clone(),
    };

    let signing_keypairs = match config.use_external_fee_pay {
        true => vec![&payer_kp, &wallet],
        false => vec![&wallet],
    };

    let instructions = match get_ixs(
        provider,
        version_number,
        proof,
        allocation,
        &wallet_pubkey,
        &cex_pubkey,
        &payer_kp.pubkey(),
        config,
    )
    .await?
    {
        Some(ixs) => ixs,
        None => return Ok(()),
    };

    let (recent_blockhash, _) = provider
        .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
        .await?;

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer_kp.pubkey()),
        &signing_keypairs,
        recent_blockhash,
    );

    send_and_confirm_tx(provider, tx, &recent_blockhash).await?;

    Ok(())
}
