use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct ClaimArgs {
    pub program_id: Pubkey,
    pub distributor: Pubkey,
    pub mint_token: Pubkey,
    pub claim_status: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub claimant: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub allocation: u64,
    pub proof: Vec<[u8; 32]>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct ClaimInput {
    allocation: u64,
    proof: Vec<[u8; 32]>,
}

impl ClaimInput {
    pub fn new(allocation: u64, proof: Vec<[u8; 32]>) -> Self {
        Self { allocation, proof }
    }
}

pub struct CreateAtaArgs {
    pub funding_address: Pubkey,
    pub associated_account_address: Pubkey,
    pub wallet_address: Pubkey,
    pub token_mint_address: Pubkey,
    pub token_program_id: Pubkey,
    pub instruction: u8,
}
