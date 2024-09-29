use crate::processor::{process_claim, process_initialize, Instructions};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

mod processor;
mod state;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = Instructions::try_from_slice(instruction_data)?;

    match instruction {
        Instructions::Initialize => process_initialize(program_id, accounts),
        Instructions::Claim { decimal } => process_claim(program_id, accounts, decimal),
    }
}

// lib
// entrypoint
// instruction
// processor
// state
// error

// TODO!
//  INSTRUCTION
//  - init
//      : create state account
//      STATE
//          : height
//          : depth
//          : value
//          : last_slot

//  - claim
//      : NOTE :: There could be multiple versions of this implementation of the mint/distribution algorithm
//      : one token is allocated per claim, any one can claim,
//      : starts off as a free claim, for each claim with in a certain time the price doubles
//      : if a claim is not executed with in a time frame the price decrease in half with a 1% offset
//      : height starts at 0, increases by 1, very every successive claim, with in time frame
//      : decreases by 1 when claim beyound time frame
//      : when height is at 0 value is 0, which the token is free to grab

//      STATE
//          : height
//          : depth
//          : value
//          : last_slot

//      INPUT
//      ACCOUNTS
//          : Signer -> User
//          : SPL_Token_Program_2022 + Extension?
//          : SPL_USDC_Token_Program
//          : USDC_Token_Mint
//          : User_USDC_Token_Account_ATA
//          : User_Claim_Token_Accunt_ATA
//          : Program_Token_Mint
//          : Program_USDC_Token_Account
//          : Program_Escrow_Token_Account? || mint tokens very every claim,
//          : Program_Pool_Token_Account?
//          : User_Metric_Account?
//          : Claim_State_Account
