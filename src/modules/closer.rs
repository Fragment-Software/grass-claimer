use std::time::Duration;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey,
    signature::Keypair, signer::Signer, transaction::Transaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    onchain::{
        constants::{GRASS_PUBKEY, TOKEN_PROGRAM_ID},
        derive::derive_ata,
        ixs::Instructions,
        tx::send_and_confirm_tx,
    },
    utils::misc::pretty_sleep,
};

pub async fn close_accounts(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) = db.get_random_account_with_filter(|a| !a.get_closed_ata()) {
        if let Err(e) = process_account(&provider, account, config).await {
            tracing::error!("{}", e);
        } else {
            account.set_closed_ata(true);
            db.update();
        };

        pretty_sleep(config.claim_sleep_range).await;
    }

    Ok(())
}

async fn get_ixs(
    provider: &RpcClient,
    wallet_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
) -> eyre::Result<Option<Vec<Instruction>>> {
    let mut ixs = vec![];

    let (wallet_token_ata, _) = derive_ata(wallet_pubkey, &GRASS_PUBKEY, &TOKEN_PROGRAM_ID);
    let token_ata_exist = provider.get_account_data(&wallet_token_ata).await.is_ok();

    if !token_ata_exist {
        tracing::warn!("Grass ATA already closed or not exist");
        return Ok(None);
    }

    let token_account = provider
        .get_token_account_balance(&wallet_token_ata)
        .await?;

    let token_account_balance = token_account.amount.parse::<u64>()?;

    if token_account_balance != 0 {
        tracing::warn!("Grass token account balance should be 0");
        return Ok(None);
    }

    let rent = provider.get_minimum_balance_for_rent_exemption(165).await?;

    let close_ix =
        Instructions::close_account(&wallet_token_ata, wallet_pubkey, payer_pubkey, rent);

    ixs.extend_from_slice(&close_ix);

    Ok(Some(ixs))
}

async fn process_account(
    provider: &RpcClient,
    account: &mut Account,
    config: &Config,
) -> eyre::Result<()> {
    let wallet = account.keypair();
    let wallet_pubkey = account.get_pubkey();

    tracing::info!("Wallet address: `{}`", wallet.pubkey());

    let payer_kp = match config.use_external_fee_pay {
        true => Keypair::from_base58_string(&config.external_fee_payer_pk),
        false => wallet.insecure_clone(),
    };

    let signing_keypairs = match config.use_external_fee_pay {
        true => vec![&payer_kp, &wallet],
        false => vec![&wallet],
    };

    let instructions = match get_ixs(provider, &wallet_pubkey, &payer_kp.pubkey()).await? {
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
