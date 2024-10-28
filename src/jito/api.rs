use std::time::Duration;

use reqwest::Proxy;
use solana_client::rpc_client::SerializableTransaction;
use tokio::time;

use crate::utils::fetch::{send_http_request, RequestParams};

use super::{
    constants::BUNDLE_ENDPOINT,
    schemas::{
        JsonRpcRequest, JsonRpcResponse, JsonRpcResponseResult, Method,
        TransactionConfirmationStatus,
    },
    utils::encode_transaction_to_bs58,
};

pub async fn send_and_confirm_bundle(
    bundle: impl SerializableTransaction,
    block_engine_url: &str,
    timeout: Option<u64>,
    poll_interval: Option<u64>,
    proxy: Option<&Proxy>,
) -> eyre::Result<String> {
    let bundle_id = send_bundle(bundle, block_engine_url, proxy).await?;
    let timeout = Duration::from_secs(timeout.unwrap_or(100));
    let poll_interval = Duration::from_secs(poll_interval.unwrap_or(5));
    confirm_bundle(bundle_id, block_engine_url, timeout, poll_interval, proxy).await
}

pub async fn send_bundle<S: SerializableTransaction>(
    bundle: S,
    block_engine_url: &str,
    proxy: Option<&Proxy>,
) -> eyre::Result<String> {
    let url = format!("{}{}", block_engine_url, BUNDLE_ENDPOINT);

    let transaction = encode_transaction_to_bs58(&bundle);

    let body = JsonRpcRequest::new(Method::SendBundle, [vec![transaction]; 1]);

    let request_params = RequestParams {
        url: &url,
        method: reqwest::Method::POST,
        body: Some(body),
        query_args: None,
        proxy,
        headers: None,
    };

    let response = send_http_request::<JsonRpcResponse>(request_params).await?;

    let JsonRpcResponseResult::SendBundleResult(bundle_id) = response.result else {
        panic!("Unexpected response result")
    };

    Ok(bundle_id)
}

async fn confirm_bundle(
    bundle_id: String,
    block_engine_url: &str,
    timeout_duration: Duration,
    poll_interval: Duration,
    proxy: Option<&Proxy>,
) -> eyre::Result<String> {
    let body = JsonRpcRequest::new(Method::GetBundleStatuses, [vec![bundle_id.clone()]; 1]);

    async fn check_confirmation_status<'a>(
        body: &JsonRpcRequest,
        block_engine_url: &str,
        poll_interval: Duration,
        bundle_id: String,
        proxy: Option<&Proxy>,
    ) -> eyre::Result<String> {
        let url = format!("{}{}", block_engine_url, BUNDLE_ENDPOINT);

        let request_params = RequestParams {
            url: &url,
            method: reqwest::Method::POST,
            body: Some(body),
            query_args: None,
            proxy,
            headers: None,
        };

        loop {
            match send_http_request::<JsonRpcResponse>(request_params.clone()).await {
                Ok(response) => match response.result {
                    JsonRpcResponseResult::GetBundleStatuses(result) => {
                        if let Some(Some(val)) = result.value.first() {
                            if let TransactionConfirmationStatus::Finalized =
                                val.confirmation_status
                            {
                                return Ok(bundle_id);
                            }
                        }
                    }
                    _ => {
                        return Err(eyre::eyre!("Unexpected response result"));
                    }
                },

                Err(err) => return Err(eyre::eyre!("Failed to send request: {}", err)),
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    match time::timeout(
        timeout_duration,
        check_confirmation_status(&body, block_engine_url, poll_interval, bundle_id, proxy),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => eyre::bail!("Timeout exceeded while confirming bundle"),
    }
}
