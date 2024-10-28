use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    SendBundle,
    GetBundleStatuses,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::SendBundle => write!(f, "sendBundle"),
            Method::GetBundleStatuses => write!(f, "getBundleStatuses"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    pub method: Method,
    params: [Vec<String>; 1],
}

impl JsonRpcRequest {
    pub fn new(method: Method, params: [Vec<String>; 1]) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method,
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    pub result: JsonRpcResponseResult,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum JsonRpcResponseResult {
    SendBundleResult(String),
    GetBundleStatuses(GetBundleStatusesResult),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBundleStatusesResult {
    context: Context,
    pub value: Vec<Option<Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    slot: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TransactionConfirmationStatus {
    Processed,
    Confirmed,
    Finalized,
}

impl Display for TransactionConfirmationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TransactionConfirmationStatus::Processed => "processed",
            TransactionConfirmationStatus::Confirmed => "confirmed",
            TransactionConfirmationStatus::Finalized => "finalized",
        };

        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Value {
    bundle_id: String,
    transactions: Vec<String>,
    slot: u64,
    pub confirmation_status: TransactionConfirmationStatus,
    err: GetBundleStatusesResultErrValue,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetBundleStatusesResultErrValue {
    #[serde(rename = "Ok")]
    ok: Option<serde_json::Value>,
}
