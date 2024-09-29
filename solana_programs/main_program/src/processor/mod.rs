use crate::state::StateAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token_2022::instruction::{
    // mint_to_checked,
    transfer_checked,
};

pub enum ErrorCode {
    AccountNeedsToBeSigner,
    Immutable,
    InvalidStateAccount,
    // AccountAlreadyIntialized,
    // AccountNotExecutable,
    // InvalidSystemProgram,
    InvalidAccountType,
    // Invalid,
    // InvalidMintAuthority,
    // InvalidTokenMint,
    // InvalidTokenProgram,
}

const STATE_SEED: &str = "state";

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum Instructions {
    Initialize,
    Claim { decimal: u8 },
    // MintTokens,
    // TokenTransfer,
    // Execute { amount: u64 },
    // InitializeExtraAccountMetaList,
    // UpdateExtraAccountMetaList,
}

pub fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let signer = next_account_info(accounts_iter)?;
    let state_account = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter)?;

    let seeds = &[STATE_SEED.as_bytes()];
    let (_pubkey, state_bump) = Pubkey::find_program_address(seeds, program_id);

    // if state_account.key != &account {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidStateAccount as u32));
    // };

    // if !state_account.data_is_empty() {
    //     return Err(ProgramError::Custom(
    //         ErrorCode::AccountAlreadyIntialized as u32,
    //     ));
    // }

    // if !signer.is_signer {
    //     return Err(ProgramError::Custom(
    //         ErrorCode::AccountNeedsToBeSigner as u32,
    //     ));
    // };

    // if !signer.is_writable {
    //     return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    // }

    // if !state_account.is_writable {
    //     return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    // }

    // if !system_program.executable {
    //     return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
    // }

    // some reason I can't get this to work properly
    // if &solana_program::system_program::ID == system_program.key {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidSystemProgram as u32));
    // }

    // using system_program that is passed in, want to use from dependency, but doesn't work
    // if authority.owner != system_program.key {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidMintAuthority as u32));
    // }

    let clock = Clock::get()?;

    let account_data = StateAccount {
        discriminator: StateAccount::DISCRIMINATOR as u8,
        bump: state_bump,

        prev_height: 0,
        last_height: 0,
        next_height: 0,
        accummulated_depth: 0,

        last_value: 0,
        next_value: 0,
        last_slot: clock.slot,

        total_claimed: 0,
        total_contributions: 0,
    };

    let size = StateAccount::LEN;
    let lamports = (Rent::get()?).minimum_balance(size);

    // create the state account
    invoke_signed(
        &create_account(
            signer.key,
            state_account.key,
            lamports,
            size as u64,
            program_id,
        ),
        &[signer.clone(), state_account.clone()],
        &[&[STATE_SEED.as_bytes(), &[state_bump]]],
    )?;

    account_data.serialize(&mut *state_account.data.borrow_mut())?;

    msg!("State Account Initialize!");

    ProgramResult::Ok(())
}

pub fn process_claim(program_id: &Pubkey, accounts: &[AccountInfo], decimal: u8) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let signer = next_account_info(accounts_iter)?;
    let source = next_account_info(accounts_iter)?;
    let _receiver = next_account_info(accounts_iter)?;

    let _authority = next_account_info(accounts_iter)?;
    let state_account = next_account_info(accounts_iter)?;

    let funding_escrow = next_account_info(accounts_iter)?;
    let _pool_escrow = next_account_info(accounts_iter)?;

    let token_mint = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let _token_hook_program = next_account_info(accounts_iter)?;

    if !signer.is_signer {
        return Err(ProgramError::Custom(
            ErrorCode::AccountNeedsToBeSigner as u32,
        ));
    };

    if !signer.is_writable {
        return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    }

    if !state_account.is_writable {
        return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    }

    let seeds = &[STATE_SEED.as_bytes()];
    let (account, _) = Pubkey::find_program_address(seeds, program_id);

    if state_account.key != &account {
        return Err(ProgramError::Custom(ErrorCode::InvalidStateAccount as u32));
    };

    let mut account_data = StateAccount::try_from_slice(&mut *state_account.data.borrow_mut())?;

    if account_data.discriminator != StateAccount::DISCRIMINATOR as u8 {
        return Err(ProgramError::Custom(ErrorCode::InvalidAccountType as u32));
    }

    let (amount, _claim_mint, _pool_mint) = account_data.update()?;

    if amount > 0 {
        let instruction = transfer_checked(
            token_program.key,
            source.key,
            // token mint must be USDC
            token_mint.key,
            // desitnation is the funding vault | escrow
            funding_escrow.key,
            signer.key,
            // probably not the best way to handle signer pubkeys, need to dynamically include them if any exist
            &[],
            amount,
            decimal,
        )?;

        let account_infos = &[
            source.clone(),
            token_mint.clone(),
            funding_escrow.clone(),
            signer.clone(),
        ];

        invoke(&instruction, account_infos)?;
    }

    // if pool_mint > 0
    // call token hook program to mint to pool escrow address

    // call token hook program to mint to signer receiving address

    Ok(())
}

// process_take_pool
