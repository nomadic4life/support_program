use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
// use spl_token::instruction::initialize_mint2;
use spl_token_2022::{instruction::initialize_mint2, state};

const STATE_SEED: &str = "state";
const TOKEN_MINT_SEED: &str = "token-mint";
const TOKEN_AUTHORITY_SEED: &str = "token-authority";

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

    pub prev_height: u8,
    pub last_height: u8,
    pub next_height: u8,

    pub last_value: u64,
    pub next_value: u64,
    pub last_slot: u64,
}

impl StateAccount {
    pub const LEN: usize = 1 * 5 + 8 * 3;
    pub const DISCRIMINATOR: usize = 0;
    pub const TARGET: usize = 60 * 1000 * 2 / 400;
    pub const RESET: u64 = 0;
    pub const DEFUALT_AMOUNT: u64 = 1;
    pub const INIT_VALUE: u64 = 1_000_000;

    // there is a potential bug when USDC value is 0.000_001
    pub fn claim(&mut self, amount: u64) -> ProgramResult {
        let clock = Clock::get()?;
        let distance = clock.slot - self.last_slot;
        let depth = distance / StateAccount::TARGET as u64;

        let multiplier = if self.next_value <= 10 { 4 } else { 2 };
        let scale: u8 = if self.next_value <= 10 { 2 } else { 1 };
        let off_set = if self.next_value <= 10 { 1 } else { 0 };
        // let divisor = if self.next_value < 100 { 1000 } else { 100 };

        let (
            // EXTRACTED VALUES
            current_value,
            current_height,
            next_value,
            next_height,
            transfer_amount,
            pool_amount,
        ) = if self.next_height == 0 {
            let next_height = self.next_height + 1;
            let next_value = StateAccount::INIT_VALUE;
            let pool_amount = 0;

            (
                StateAccount::RESET,
                self.next_height,
                next_value,
                next_height,
                StateAccount::DEFUALT_AMOUNT,
                pool_amount,
            )
        } else if depth == 0 {
            let next_value = self.next_value * multiplier;
            let next_height = self.next_height + 1;
            let pool_amount = 0;

            (
                self.next_value,
                self.next_height,
                next_value,
                next_height,
                StateAccount::DEFUALT_AMOUNT,
                pool_amount,
            )
        } else if self.next_height > depth as u8 {
            let value = (self.next_value >> (depth * scale as u64)) - off_set;
            let current_value = value + value * depth / 100;
            let current_height = self.next_height - depth as u8;
            let next_height = current_height + 1;
            let next_value = value * 2;
            let transfer_amount = if amount > depth { depth } else { amount };
            let pool_amount = depth - transfer_amount;

            (
                current_value,
                current_height,
                next_value,
                next_height,
                transfer_amount,
                pool_amount,
            )
        } else {
            let value = (self.next_value >> (self.next_height * scale)) - off_set;
            let current_value = 0;
            let next_value = value + value * depth / 100;
            let current_height = 0;
            let next_height = 1;
            let transfer_amount = if amount > depth { depth } else { amount };
            let pool_amount = depth - transfer_amount;

            (
                current_value,
                current_height,
                next_value,
                next_height,
                transfer_amount,
                pool_amount,
            )
        };

        self.last_slot = clock.slot;
        self.last_height = current_height;
        self.last_value = current_value;

        self.next_height = next_height;
        self.next_value = next_value;
        self.prev_height = self.next_height;

        // if pool_amount != 0
        // transfer / mint pool amount to pool

        // if last_value != 0
        // transfer current value

        // transfer claim token

        Ok(())
    }
}

pub enum ErrorCode {
    AccountNeedsToBeSigner,
    Immutable,
    InvalidStateAccount,
    AccountAlreadyIntialized,
    AccountNotExecutable,
    InvalidSystemProgram,
    InvalidAccountType,
    Invalid,
    InvalidMintAuthority,
    InvalidTokenMint,
    InvalidTokenProgram,
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

    let seeds = &[STATE_SEED.as_bytes()];
    let (account, state_bump) = Pubkey::find_program_address(seeds, program_id);

    if state_account.key != &account {
        return Err(ProgramError::Custom(ErrorCode::InvalidStateAccount as u32));
    };

    if !state_account.data_is_empty() {
        return Err(ProgramError::Custom(
            ErrorCode::AccountAlreadyIntialized as u32,
        ));
    }

    let token_program = next_account_info(accounts_iter)?;
    if !token_program.executable {
        return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
    }

    // why this doesn't work?
    // if token_program.key == &spl_token_2022::ID {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidTokenProgram as u32));
    // }

    let token_mint = next_account_info(accounts_iter)?;
    if !token_mint.is_writable {
        return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    }

    let seeds = &[TOKEN_MINT_SEED.as_bytes()];
    let (account, mint_bump) = Pubkey::find_program_address(seeds, program_id);
    if token_mint.key != &account {
        return Err(ProgramError::Custom(ErrorCode::InvalidTokenMint as u32));
    };

    let authority = next_account_info(accounts_iter)?;

    let seeds = &[TOKEN_AUTHORITY_SEED.as_bytes()];
    let (account, _) = Pubkey::find_program_address(seeds, program_id);

    if authority.key != &account {
        return Err(ProgramError::Custom(ErrorCode::InvalidMintAuthority as u32));
    };

    let system_program = next_account_info(accounts_iter)?;
    if !system_program.executable {
        return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
    }

    // some reason I can't get this to work properly
    // if &solana_program::system_program::ID == system_program.key {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidSystemProgram as u32));
    // }

    let clock = Clock::get()?;

    let account_data = StateAccount {
        discriminator: StateAccount::DISCRIMINATOR as u8,
        bump: state_bump,
        prev_height: 0,
        last_height: 0,
        next_height: 0,

        last_value: 0,
        next_value: 0,
        last_slot: clock.slot,
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

    // adding 152 for the extension + but why 152?
    let size = state::Mint::LEN + 152;
    let lamports = (Rent::get()?).minimum_balance(size);

    // create the account -> mint_token
    invoke_signed(
        &create_account(
            signer.key,
            token_mint.key,
            lamports,
            size as u64,
            token_program.key,
        ),
        &[signer.clone(), token_mint.clone()],
        &[&[TOKEN_MINT_SEED.as_bytes(), &[mint_bump]]],
    )?;

    // create the token hook relation
    invoke(
        &spl_token_2022::extension::transfer_hook::instruction::initialize(
            &token_program.key,
            token_mint.key,
            None,
            Some(program_id.clone()),
        )?,
        &[token_mint.clone()],
    )?;

    // initialize the token_mint
    let decimals = 9;
    invoke(
        &initialize_mint2(
            &token_program.key,
            &token_mint.key,
            &authority.key,
            Some(&authority.key),
            decimals,
        )?,
        &[token_mint.clone()],
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

    let seeds = &[STATE_SEED.as_bytes()];
    let (account, _) = Pubkey::find_program_address(seeds, program_id);

    if state_account.key != &account {
        return Err(ProgramError::Custom(ErrorCode::InvalidStateAccount as u32));
    };

    let mut account_data = StateAccount::try_from_slice(&mut *state_account.data.borrow_mut())?;

    if account_data.discriminator != StateAccount::DISCRIMINATOR as u8 {
        return Err(ProgramError::Custom(ErrorCode::InvalidAccountType as u32));
    }

    let amount = 1;
    account_data.claim(amount)
}

pub fn initialize_hook(
    token_program_id: &Pubkey,
    mint_info: &AccountInfo,
    authority: Option<Pubkey>,
    transfer_hook_program_id: Option<Pubkey>,
) -> ProgramResult {
    invoke(
        &spl_token_2022::extension::transfer_hook::instruction::initialize(
            token_program_id,
            mint_info.key,
            authority,
            transfer_hook_program_id,
        )?,
        &[mint_info.clone()],
    )
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
