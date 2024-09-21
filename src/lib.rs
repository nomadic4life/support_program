use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = Instructions::try_from_slice(instruction_data)?;

    match instruction {
        Instructions::Initialize => process_initialize_state(program_id, accounts),
        Instructions::Claim => process_claim(program_id, accounts),
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Instructions {
    Initialize,
    Claim,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StateAccount {
    pub discriminator: u8,
    pub bump: u8,
    pub height: u8,
    pub last_depth: u8,
    pub last_value: u64,
    pub last_slot: u64,
}

impl StateAccount {
    pub const LEN: usize = 1 * 4 + 8 * 2;
    pub const DISCRIMINATOR: usize = 0;

    pub fn claim(&mut self) -> ProgramResult {
        let clock = Clock::get()?;

        self.height += 1;
        self.last_depth = 0;
        self.last_value += self.last_value * 2;
        self.last_slot = clock.slot;

        Ok(())
    }
}

pub enum ErrorCode {
    AccountNeedsToBeSigner,
    Immutable,
    InvalidStateAccount,
    AccountAlreadyIntialized,
    AccountNotExecutable,
    InvalidSystemAccount,
    InvalidAccountType,
}

pub fn process_initialize_state(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let signer = next_account_info(accounts_iter)?;
    if !signer.is_signer {
        return Err(ProgramError::Custom(
            ErrorCode::AccountNeedsToBeSigner as u32,
        ));
    };

    if !signer.is_writable {
        return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    }

    let state_account = next_account_info(accounts_iter)?;
    if !state_account.is_writable {
        return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    }

    let seeds = &["state".as_bytes()];
    let (account, bump) = Pubkey::find_program_address(seeds, program_id);

    if state_account.key != &account {
        return Err(ProgramError::Custom(ErrorCode::InvalidStateAccount as u32));
    };

    if !state_account.data_is_empty() {
        return Err(ProgramError::Custom(
            ErrorCode::AccountAlreadyIntialized as u32,
        ));
    }

    let system_program = next_account_info(accounts_iter)?;
    if !system_program.executable {
        return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
    }

    if solana_program::system_program::check_id(system_program.key) {
        return Err(ProgramError::Custom(ErrorCode::InvalidSystemAccount as u32));
    }

    let size = StateAccount::LEN;
    let lamports = (Rent::get()?).minimum_balance(size);

    let clock = Clock::get()?;

    let account_data = StateAccount {
        discriminator: StateAccount::DISCRIMINATOR as u8,
        bump,
        height: 0,
        last_depth: 0,
        last_value: 0,
        last_slot: clock.slot,
    };

    invoke(
        &create_account(
            signer.key,
            state_account.key,
            lamports,
            size as u64,
            program_id,
        ),
        &[
            signer.clone(),
            state_account.clone(),
            system_program.clone(),
        ],
    )?;

    account_data.serialize(&mut *state_account.data.borrow_mut())?;

    msg!("State Account Initialize!");

    ProgramResult::Ok(())
}

pub fn process_claim(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let signer = next_account_info(accounts_iter)?;
    if !signer.is_signer {
        return Err(ProgramError::Custom(
            ErrorCode::AccountNeedsToBeSigner as u32,
        ));
    };

    if !signer.is_writable {
        return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    }

    let state_account = next_account_info(accounts_iter)?;
    if !state_account.is_writable {
        return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    }

    let seeds = &["state".as_bytes()];
    let (account, _) = Pubkey::find_program_address(seeds, program_id);

    if state_account.key != &account {
        return Err(ProgramError::Custom(ErrorCode::InvalidStateAccount as u32));
    };

    let mut account_data = StateAccount::try_from_slice(&mut *state_account.data.borrow_mut())?;

    if account_data.discriminator != StateAccount::DISCRIMINATOR as u8 {
        return Err(ProgramError::Custom(ErrorCode::InvalidAccountType as u32));
    }

    account_data.claim()
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
