use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct GrassApiResponse<T> {
    pub result: Option<T>,
}

#[derive(Deserialize)]
pub struct Allocation {
    #[serde(flatten)]
    pub dynamic_fields: HashMap<String, f64>,
}

#[derive(Deserialize, Debug)]
pub struct ReceiptData {
    #[serde(rename = "versionNumber")]
    pub version_number: Option<u32>,
    #[serde(rename = "claimProof")]
    pub claim_proof: Option<String>, // json string
}

#[derive(Deserialize, Debug)]
pub struct Receipt {
    pub data: Option<ReceiptData>,
}

#[derive(Deserialize, Debug)]
pub struct ClaimProofEntry {
    pub data: InnerData,
}

#[derive(Deserialize, Debug)]
pub struct InnerData {
    pub data: String,
}
