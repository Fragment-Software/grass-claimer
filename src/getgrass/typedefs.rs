use serde::Serialize;

pub enum Cluster {
    Mainnet,
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Cluster::Mainnet => "mainnet",
        };
        write!(f, "{}", s)
    }
}
#[derive(Serialize)]
pub struct AllocationQuery {
    #[serde(rename = "walletAddress")]
    wallet_address: String,
}

impl AllocationQuery {
    pub fn to_string(wallet_address: &str) -> eyre::Result<String, serde_json::Error> {
        let query = Self {
            wallet_address: wallet_address.to_string(),
        };

        serde_json::to_string(&query)
    }
}

#[derive(Serialize)]
pub struct ReceiptQuery {
    #[serde(rename = "walletAddress")]
    wallet_address: String,
    cluster: String,
}

impl ReceiptQuery {
    pub fn to_string(
        wallet_address: &str,
        cluster: Cluster,
    ) -> eyre::Result<String, serde_json::Error> {
        let query = Self {
            wallet_address: wallet_address.to_string(),
            cluster: cluster.to_string(),
        };

        serde_json::to_string(&query)
    }
}
