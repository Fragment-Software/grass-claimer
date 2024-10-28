use solana_sdk::pubkey::Pubkey;

use super::constants::{ASSOCIATED_TOKEN_PROGRAM_ID, CLAIM_PROGRAM_ID, GRASS_PUBKEY};

pub fn derive_ata(user: &Pubkey, token_mint: &Pubkey, token_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &user.to_bytes(),
            &token_program_id.to_bytes(),
            &token_mint.to_bytes(),
        ],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
}

pub fn derive_merkle_distributor(version_number: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"MerkleDistributor",
            &GRASS_PUBKEY.to_bytes(),
            &version_number.to_le_bytes(),
        ],
        &CLAIM_PROGRAM_ID,
    )
}

pub fn derive_claim_status(user: &Pubkey, merkle_distributor: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"ClaimStatus",
            &user.to_bytes(),
            &merkle_distributor.to_bytes(),
        ],
        &CLAIM_PROGRAM_ID,
    )
}
