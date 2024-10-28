use reqwest::{Method, Proxy};

use crate::utils::fetch::{send_http_request, RequestParams};

use super::{
    constants::{AIRDROP_ELIGIBILITY_CHECKER, CLAIM_AIRDROP_RECEIPT},
    schemas::{Allocation, GrassApiResponse, Receipt},
    typedefs::{AllocationQuery, Cluster, ReceiptQuery},
};

pub async fn get_allocation(
    wallet_address: &str,
    proxy: Option<&Proxy>,
) -> eyre::Result<GrassApiResponse<Allocation>> {
    let query =
        AllocationQuery::to_string(wallet_address).expect("Failed to stringify allocation query");

    let query_args = [("input", query.as_str())].into_iter().collect();

    let request_params = RequestParams {
        url: AIRDROP_ELIGIBILITY_CHECKER,
        method: Method::GET,
        body: None::<serde_json::Value>,
        query_args: Some(query_args),
        proxy,
        headers: None,
    };

    let response_body = send_http_request::<GrassApiResponse<Allocation>>(request_params).await?;

    Ok(response_body)
}

pub async fn get_receipt(
    wallet_address: &str,
    cluster: Cluster,
    proxy: Option<&Proxy>,
) -> eyre::Result<GrassApiResponse<Receipt>> {
    let query = ReceiptQuery::to_string(wallet_address, cluster)
        .expect("Failed to stringify receipt query");

    let query_args = [("input", query.as_str())].into_iter().collect();

    let request_params = RequestParams {
        url: CLAIM_AIRDROP_RECEIPT,
        method: Method::GET,
        body: None::<serde_json::Value>,
        query_args: Some(query_args),
        proxy,
        headers: None,
    };

    let response_body = send_http_request::<GrassApiResponse<Receipt>>(request_params).await?;

    Ok(response_body)
}
