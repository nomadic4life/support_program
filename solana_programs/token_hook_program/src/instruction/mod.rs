// use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::{
    // convert::{TryFrom, TryInto},
    mem::size_of,
};

// #[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TokenInstruction {
    MintTo {
        /// The amount of new tokens to mint.
        amount: u64,
    },
}

impl TokenInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::MintTo { amount } => {
                buf.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        };
        buf
    }
}

pub fn mint_to(
    token_hook_program_id: &Pubkey,
    token_program_id: &Pubkey,
    token_mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let data = TokenInstruction::MintTo { amount }.pack();

    let mut accounts = Vec::with_capacity(4);
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new(*token_mint_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*mint_authority_pubkey, true));
    accounts.push(AccountMeta::new_readonly(*token_program_id, false));

    Ok(Instruction {
        program_id: *token_hook_program_id,
        accounts,
        data,
    })
}
