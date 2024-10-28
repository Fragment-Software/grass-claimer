use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(BorshDeserialize)]
pub struct ClaimStatus {
    _claimant: Pubkey,
    pub allocation: u64,
    pub sent_allocation: u64,
    _claimed_ts: i64,
}
