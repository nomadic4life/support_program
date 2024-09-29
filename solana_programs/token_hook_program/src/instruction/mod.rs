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
                buf.push(0);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        };
        buf
    }
}

pub fn mint_to(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    // token_mint: &AccountInfo,
    // receipent: &AccountInfo,
    // authority: &AccountInfo,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let data = TokenInstruction::MintTo { amount }.pack();

    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new(*mint_pubkey, false));
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*owner_pubkey, true));

    Ok(Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    })
}
