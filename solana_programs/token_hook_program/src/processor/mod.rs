use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_tlv_account_resolution::{
    // account::ExtraAccountMeta,
    state::ExtraAccountMetaList,
};
use spl_token_2022::{
    instruction::{initialize_mint2, mint_to_checked},
    state,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use std::vec;

const TOKEN_MINT_SEED: &str = "token-mint";
const TOKEN_AUTHORITY_SEED: &str = "token-authority";

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum Instructions {
    InitializeTokenMint,
    MintTokens { amount: u64 },
    Execute { amount: u64 },
    InitializeExtraAccountMetaList,
    UpdateExtraAccountMetaList,
}

pub fn process_initialize_token_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;

    let seeds = &[TOKEN_MINT_SEED.as_bytes()];
    let (_account, mint_bump) = Pubkey::find_program_address(seeds, program_id);

    let seeds = &[TOKEN_AUTHORITY_SEED.as_bytes()];
    let (_account, _) = Pubkey::find_program_address(seeds, program_id);

    // if !token_program.executable {
    //     return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
    // }

    // why this doesn't work?
    // if token_program.key == &spl_token_2022::ID {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidTokenProgram as u32));
    // }

    // if !token_mint.is_writable {
    //     return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    // }

    // if token_mint.key != &account {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidTokenMint as u32));
    // };

    // if authority.key != &account {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidMintAuthority as u32));
    // };

    // adding 152 for the extension + but why 152?
    let size = state::Mint::LEN + 152;
    // let size = state::Mint::LEN;
    let lamports = (Rent::get()?).minimum_balance(size);

    // create the account -> mint_token
    invoke_signed(
        &create_account(
            payer.key,
            token_mint.key,
            lamports,
            size as u64,
            token_program.key,
        ),
        &[payer.clone(), token_mint.clone()],
        &[&[TOKEN_MINT_SEED.as_bytes(), &[mint_bump]]],
    )?;

    // create the token hook relation
    invoke(
        &spl_token_2022::extension::transfer_hook::instruction::initialize(
            token_program.key,
            token_mint.key,
            Some(authority.key.clone()),
            Some(program_id.clone()),
        )?,
        &[token_mint.clone()],
    )?;

    // initialize the token_mint
    let decimals = 9;
    invoke(
        &initialize_mint2(
            token_program.key,
            token_mint.key,
            authority.key,
            Some(authority.key),
            decimals,
        )?,
        &[token_mint.clone()],
    )?;
    Ok(())
}

// // curently working on
pub fn process_execute(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _amount: u64,
) -> ProgramResult {
    // source
    // destination

    // transfer 1
    // any source -> program escrow destination
    //  tax is applied to amount and remains in escrow
    //  remaining is recorded to the intended receipent

    //  transfer 2
    //  program escrow source -> tax vault destination
    //  must be executed before final transaction
    //  anyone can exuecte transaction, done from program

    // transfer 3
    //  program escrow source -> receipent destenation
    //  final transaction
    //  anyone can execute transaction, done from program

    Ok(())
}

pub fn process_mint_tokens(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // let _signer = next_account_info(accounts_iter)?;
    let receipent = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // let seeds = &[TOKEN_AUTHORITY_SEED.as_bytes()];
    // let (_account, bump) = Pubkey::find_program_address(seeds, program_id);

    // if authority.key != &_account {
    //     return Err(ProgramError::Custom(ErrorCode::InvalidMintAuthority as u32));
    // };

    // if !token_program.executable {
    //     return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
    // }

    // if !token_mint.is_writable {
    //     return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
    // }

    let decimals = 9;
    invoke(
        &mint_to_checked(
            token_program.key,
            token_mint.key,
            receipent.key,
            // authority is the mint autority
            authority.key,
            &[],
            amount,
            decimals,
        )?,
        &[token_mint.clone(), receipent.clone(), authority.clone()][..],
    )?;

    // invoke_signed(
    //     &mint_to_checked(
    //         token_program.key,
    //         token_mint.key,
    //         receipent.key,
    //         authority.key,
    //         &[],
    //         amount,
    //         decimals,
    //     )?,
    //     &[token_mint.clone(), receipent.clone(), authority.clone()][..],
    //     &[&[TOKEN_AUTHORITY_SEED.as_bytes(), &[bump][..]]],
    // )

    Ok(())
}

pub fn process_initialize_extra_account_meta_list(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    let extra_account_meta_list = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter);

    // Here will add all the account metas
    let account_metas = vec![];
    let account_size = ExtraAccountMetaList::size_of(account_metas.len())? as u64;
    let lamports = (Rent::get()?).minimum_balance(account_size as usize);

    let (_pubkey, bump) = Pubkey::find_program_address(
        &[b"extra-account-metas", token_mint.key.as_ref()],
        program_id,
    );

    invoke_signed(
        &create_account(
            payer.key,
            extra_account_meta_list.key,
            lamports,
            account_size as u64,
            program_id,
        ),
        &[payer.clone(), extra_account_meta_list.clone()],
        &[&[b"extra-account-metas", token_mint.key.as_ref(), &[bump][..]]],
    )?;

    ExtraAccountMetaList::init::<ExecuteInstruction>(
        &mut extra_account_meta_list.try_borrow_mut_data()?,
        &account_metas,
    )?;

    Ok(())
}
