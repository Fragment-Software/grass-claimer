use std::str::FromStr;

use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Account {
    private_key: String,
    proxy: Option<String>,
    address: String,
    cex_address: String,
    allocation: f64,
    claimed: bool,
    closed_ata: bool,
    collected_sol: bool,
}

impl Account {
    pub fn new(private_key: &str, proxy: Option<String>, cex_address: &str) -> Self {
        let signer = Keypair::from_base58_string(private_key);
        let address = signer.pubkey();

        Self {
            private_key: private_key.to_string(),
            proxy,
            address: address.to_string(),
            cex_address: cex_address.to_string(),
            ..Default::default()
        }
    }

    pub fn proxy(&self) -> Option<Proxy> {
        self.proxy
            .as_ref()
            .map(|proxy| Proxy::all(proxy).expect("Proxy to be valid"))
    }

    pub fn keypair(&self) -> Keypair {
        Keypair::from_base58_string(&self.private_key)
    }

    pub fn get_pubkey(&self) -> Pubkey {
        Pubkey::from_str(&self.address).expect("Address to be valid")
    }

    pub fn set_allocation(&mut self, allocation: f64) {
        self.allocation = allocation
    }

    pub fn set_claimed(&mut self, claimed: bool) {
        self.claimed = claimed
    }

    pub fn get_claimed(&self) -> bool {
        self.claimed
    }

    pub fn get_cex_address(&self) -> &str {
        &self.cex_address
    }

    pub fn get_closed_ata(&self) -> bool {
        self.closed_ata
    }

    pub fn set_closed_ata(&mut self, closed: bool) {
        self.closed_ata = closed
    }

    pub fn get_collected_sol(&self) -> bool {
        self.collected_sol
    }

    pub fn set_collected_sol(&mut self, collected_sol: bool) {
        self.collected_sol = collected_sol
    }
}
