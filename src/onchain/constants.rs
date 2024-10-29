use std::sync::LazyLock;

use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

pub const GRASS_PUBKEY: Pubkey = pubkey!("Grass7B4RdKfBCjTKgSqnXkqjwiGvQyFbuSCUJr3XXjs");

pub static CLOSE_PUBKEY: LazyLock<Pubkey> = LazyLock::new(|| {
    Pubkey::new_from_array([
        156, 194, 179, 36, 147, 69, 5, 154, 187, 52, 104, 42, 42, 191, 111, 230, 84, 196, 250, 181,
        2, 30, 228, 228, 148, 3, 135, 142, 129, 86, 251, 163,
    ])
});

pub const CLAIM_PROGRAM_ID: Pubkey = pubkey!("Eohp5jrnGQgP74oD7ij9EuCSYnQDLLHgsuAmtSTuxABk");

pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub const INSTRUCTION_NAMESPACE: &str = "global";
