use std::{str::FromStr, time::Duration};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey,
    signature::Keypair, signer::Signer, transaction::Transaction,
};

use crate::{
    config::Config,
    db::{account::Account, database::Database},
    onchain::tx::send_and_confirm_tx,
    utils::misc::pretty_sleep,
};

pub async fn collect_sol(mut db: Database, config: &Config) -> eyre::Result<()> {
    let provider = RpcClient::new_with_timeout_and_commitment(
        config.solana_rpc_url.clone(),
        Duration::from_secs(60),
        CommitmentConfig::processed(),
    );

    while let Some(account) = db.get_random_account_with_filter(|a| !a.get_collected_sol()) {
        if let Err(e) = process_account(&provider, account, config).await {
            tracing::error!("{}", e);
        } else {
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
) -> eyre::Result<Option<Vec<Instruction>>> {
    let mut ixs = vec![];

    let balance = provider.get_balance(wallet_pubkey).await?;

    ixs.push(solana_sdk::system_instruction::transfer(
        wallet_pubkey,
        collector_pubkey,
        balance,
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

    let instructions = match get_ixs(provider, &wallet_pubkey, &collector_pubkey).await? {
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
