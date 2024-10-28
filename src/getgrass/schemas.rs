use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct GrassApiResponse<T> {
    pub result: Option<T>,
}

#[derive(Deserialize)]
pub struct Allocation {
    pub dynamic_fields: HashMap<String, f64>,
}

#[derive(Deserialize, Debug)]
pub struct ReceiptData {
    #[serde(rename = "versionNumber")]
    pub version_number: Option<u32>,
    #[serde(rename = "claimProof")]
    pub claim_proof: Option<String>, // json string
    pub allocation: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct Receipt {
    pub data: Option<ReceiptData>,
}

#[derive(Deserialize, Debug)]
pub struct ClaimProofEntry {
    pub position: String,
    pub data: BufferData,
}

#[derive(Deserialize, Debug)]
pub struct BufferData {
    #[serde(rename = "type")]
    pub type_: String,
    pub data: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct InnerData {
    pub data: String,
}
