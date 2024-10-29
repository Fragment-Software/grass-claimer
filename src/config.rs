use serde::Deserialize;
use std::path::Path;

#[allow(unused)]
const CONFIG_FILE_PATH: &str = "data/config.toml";

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Config {
    pub solana_rpc_url: String,
    pub withdraw_to_cex: bool,
    pub mobile_proxies: bool,
    pub swap_ip_link: String,
    pub claim_sleep_range: [u64; 2],
    pub use_external_fee_pay: bool,
    pub external_fee_payer_pk: String,
}

impl Config {
    async fn read_from_file(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let cfg_str = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&cfg_str)?)
    }

    pub async fn read_default() -> Self {
        Self::read_from_file(CONFIG_FILE_PATH)
            .await
            .expect("Default config to be valid")
    }
}
