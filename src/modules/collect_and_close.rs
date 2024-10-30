use std::{str::FromStr, time::Duration};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, program_pack::Pack,
    pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    onchain::{
        constants::{GRASS_PUBKEY, TOKEN_PROGRAM_ID},
        derive::derive_ata,
        ixs::Instructions,
        tx::send_and_confirm_tx,
        typedefs::CreateAtaArgs,
    },
    utils::misc::pretty_sleep,
};

pub async fn collect_and_close(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) =
        db.get_random_account_with_filter(|a| !a.get_collected_sol() || !a.get_closed_ata())
    {
        if let Err(e) = process_account(&provider, account, config).await {
            tracing::error!("{}", e);
        } else {
            account.set_closed_ata(true);
            account.set_collected_sol(true);
            db.update();
        };

        pretty_sleep(config.claim_sleep_range).await;
    }

    Ok(())
}

async fn get_ixs(
    provider: &RpcClient,
    wallet_pubkey: &Pubkey,
    collector_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
) -> eyre::Result<Option<Vec<Instruction>>> {
    let mut ixs = vec![];

    let (wallet_token_ata, _) = derive_ata(wallet_pubkey, &GRASS_PUBKEY, &TOKEN_PROGRAM_ID);
    let token_ata_exist = provider.get_account_data(&wallet_token_ata).await.is_ok();

    if token_ata_exist {
        let token_account = provider
            .get_token_account_balance(&wallet_token_ata)
            .await?;

        let token_account_balance = token_account.amount.parse::<u64>()?;

        if token_account_balance != 0 {
            let (collector_token_ata, _) =
                derive_ata(collector_pubkey, &GRASS_PUBKEY, &TOKEN_PROGRAM_ID);
            let collector_token_ata_exist = provider
                .get_account_data(&collector_token_ata)
                .await
                .is_ok();

            if !collector_token_ata_exist {
                let create_ata_args = CreateAtaArgs {
                    funding_address: *payer_pubkey,
                    associated_account_address: collector_token_ata,
                    wallet_address: *collector_pubkey,
                    token_mint_address: GRASS_PUBKEY,
                    token_program_id: TOKEN_PROGRAM_ID,
                    instruction: 0,
                };

                ixs.push(Instructions::create_ata(create_ata_args));
            }

            ixs.push(spl_token::instruction::transfer_checked(
                &TOKEN_PROGRAM_ID,
                &wallet_token_ata,
                &GRASS_PUBKEY,
                &collector_token_ata,
                wallet_pubkey,
                &[wallet_pubkey],
                token_account_balance,
                9u8,
            )?);
        }

        let rent = provider
            .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
            .await?;

        let close_ix =
            Instructions::close_account(&wallet_token_ata, wallet_pubkey, payer_pubkey, rent);

        ixs.extend_from_slice(&close_ix);
    }

    let balance = provider.get_balance(wallet_pubkey).await?;

    if balance <= 5000 {
        tracing::warn!(
            "Wallet doesn't have enough SOL to withdraw: {} | 5001 at least",
            balance
        );
        return Ok(None);
    }

    let amount_to_withdraw = if payer_pubkey == wallet_pubkey {
        balance - 5000
    } else {
        balance
    };

    ixs.push(solana_sdk::system_instruction::transfer(
        wallet_pubkey,
        collector_pubkey,
        amount_to_withdraw,
    ));

    Ok(Some(ixs))
}

async fn process_account(
    provider: &RpcClient,
    account: &mut Account,
    config: &Config,
) -> eyre::Result<()> {
    let wallet = account.keypair();
    let wallet_pubkey = account.get_pubkey();
    let collector_pubkey = Pubkey::from_str(&config.collector_pubkey)?;

    tracing::info!("Wallet address: `{}`", wallet.pubkey());

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
        &wallet_pubkey,
        &collector_pubkey,
        &payer_kp.pubkey(),
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
