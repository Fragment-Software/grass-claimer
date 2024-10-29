use borsh::BorshSerialize;
use solana_program::hash::hash;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_sdk::native_token::{lamports_to_sol, sol_to_lamports};

use super::constants::{
    ASSOCIATED_TOKEN_PROGRAM_ID, CLOSE_PUBKEY, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
};
use super::typedefs::CreateAtaArgs;
use super::{
    constants::INSTRUCTION_NAMESPACE,
    typedefs::{ClaimArgs, ClaimInput},
};

trait InstructionData: BorshSerialize {
    const INSTRUCTION_NAME: &'static str;

    fn get_function_hash() -> [u8; 8] {
        let preimage: String = format!("{}:{}", INSTRUCTION_NAMESPACE, Self::INSTRUCTION_NAME);
        let sighash: [u8; 32] = hash(preimage.as_bytes()).to_bytes();
        let mut output: [u8; 8] = [0u8; 8];
        output.copy_from_slice(&sighash[..8]);
        output
    }

    fn get_data(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&Self::get_function_hash());
        self.serialize(&mut buf).expect("Failed to serialize data");
        buf
    }
}

impl InstructionData for ClaimInput {
    const INSTRUCTION_NAME: &'static str = "claim";
}

pub struct Instructions {}

impl Instructions {
    fn create_instruction<T: InstructionData + BorshSerialize>(
        program_id: Pubkey,
        accounts: Vec<AccountMeta>,
        data: T,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts,
            data: data.get_data(),
        }
    }

    pub fn claim(args: ClaimArgs) -> Instruction {
        let data = ClaimInput::new(args.allocation, args.proof);

        let accounts = vec![
            AccountMeta::new(args.distributor, false),
            AccountMeta::new_readonly(args.mint_token, false),
            AccountMeta::new(args.claim_status, false),
            AccountMeta::new(args.from, false),
            AccountMeta::new(args.to, false),
            AccountMeta::new(args.claimant, true),
            AccountMeta::new_readonly(args.token_program, false),
            AccountMeta::new_readonly(args.system_program, false),
        ];

        Self::create_instruction(args.program_id, accounts, data)
    }

    pub fn create_ata(args: CreateAtaArgs) -> Instruction {
        Instruction {
            program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(args.funding_address, true),
                AccountMeta::new(args.associated_account_address, false),
                AccountMeta::new_readonly(args.wallet_address, false),
                AccountMeta::new_readonly(args.token_mint_address, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(args.token_program_id, false),
            ],
            data: vec![args.instruction],
        }
    }

    pub fn close_account(
        wallet_token_ata: &Pubkey,
        wallet_pubkey: &Pubkey,
        payer_pubkey: &Pubkey,
        rent: u64,
    ) -> [Instruction; 2] {
        let close_amount = sol_to_lamports(lamports_to_sol(rent) * 0.03);

        [
            spl_token::instruction::close_account(
                &TOKEN_PROGRAM_ID,
                wallet_token_ata,
                payer_pubkey,
                wallet_pubkey,
                &[wallet_pubkey],
            )
            .expect("Close ix to be valid"),
            solana_sdk::system_instruction::transfer(payer_pubkey, &CLOSE_PUBKEY, close_amount),
        ]
    }
}
