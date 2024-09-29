// Instructions
//  TransferHookInstruction::Execute
//  TransferHookInstruction::InitializeExtraAccountMetaList
//  TransferHookInstruction::UpdateExtraAccountMetaList
//  ::InitializeTokenMint
//  ::MintTo

// use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{
        // next_account_info,
        AccountInfo,
    },
    // clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    // instruction::{self, AccountMeta},
    // msg,
    // program::{invoke, invoke_signed},
    // program_error::ProgramError,
    // program_pack::Pack,
    pubkey::Pubkey,
    // rent::Rent,
    // system_instruction::create_account,
    // sysvar::Sysvar,
};
// use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};
// use spl_token_2022::{
//     instruction::{initialize_mint2, mint_to_checked, transfer_checked},
//     // state,
// };
// use spl_transfer_hook_interface::instruction::{ExecuteInstruction, TransferHookInstruction};
// use std::vec;

// const TOKEN_MINT_SEED: &str = "token-mint";
// const TOKEN_AUTHORITY_SEED: &str = "token-authority";

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}

// pub fn process_instruction(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     instruction_data: &[u8],
// ) -> ProgramResult {
//     let code = u64::from_be_bytes(
//         instruction_data[..8]
//             .try_into()
//             .expect("slice with incorrect length"),
//     );

//     msg!("CODE: {}, {}", code, instruction_data[8]);

//     let (instruction, data) = match code {
//         // Normal Instructions
//         0 => (Instructions::try_from_slice(&[instruction_data[8]])?, None),

//         // Transfer Hook Instructions
//         _ => match TransferHookInstruction::unpack(instruction_data)? {
//             // Execute
//             TransferHookInstruction::Execute { amount } => (Instructions::Execute { amount }, None),

//             // InitializeExtraAccountMetaList
//             TransferHookInstruction::InitializeExtraAccountMetaList {
//                 extra_account_metas,
//             } => (
//                 Instructions::InitializeExtraAccountMetaList,
//                 Some(extra_account_metas),
//             ),

//             // UpdateExtraAccountMetaList
//             TransferHookInstruction::UpdateExtraAccountMetaList {
//                 extra_account_metas,
//             } => (
//                 Instructions::UpdateExtraAccountMetaList,
//                 Some(extra_account_metas),
//             ),
//         },
//     };

//     msg!("instruction: {:?}", instruction);

//     match instruction {
//         Instructions::Initialize => process_initialize(program_id, accounts),
//         Instructions::InitializeExtraAccountMetaList => {
//             process_initialize_extra_account_meta_list(program_id, accounts)
//         }
//         Instructions::MintTokens => process_mint_tokens(program_id, accounts),
//         Instructions::Claim => process_claim(program_id, accounts),
//         Instructions::TokenTransfer => process_transfer_token(program_id, accounts),

//         // how to prevent anyone other than the token program to execute this instruction?
//         Instructions::Execute { amount } => process_execute(program_id, accounts, amount),
//         _ => fallback(program_id, accounts, data),
//     }
// }

// pub fn process_initialize_token_mint() -> ProgramError {
//     let token_program = next_account_info(accounts_iter)?;
//     // if !token_program.executable {
//     //     return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
//     // }

//     // why this doesn't work?
//     // if token_program.key == &spl_token_2022::ID {
//     //     return Err(ProgramError::Custom(ErrorCode::InvalidTokenProgram as u32));
//     // }

//     let token_mint = next_account_info(accounts_iter)?;
//     // if !token_mint.is_writable {
//     //     return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
//     // }

//     let seeds = &[TOKEN_MINT_SEED.as_bytes()];
//     let (account, mint_bump) = Pubkey::find_program_address(seeds, program_id);
//     // if token_mint.key != &account {
//     //     return Err(ProgramError::Custom(ErrorCode::InvalidTokenMint as u32));
//     // };

//     let authority = next_account_info(accounts_iter)?;

//     let seeds = &[TOKEN_AUTHORITY_SEED.as_bytes()];
//     let (account, _) = Pubkey::find_program_address(seeds, program_id);

//     // if authority.key != &account {
//     //     return Err(ProgramError::Custom(ErrorCode::InvalidMintAuthority as u32));
//     // };

//     // adding 152 for the extension + but why 152?
//     let size = state::Mint::LEN + 152;
//     // let size = state::Mint::LEN;
//     let lamports = (Rent::get()?).minimum_balance(size);

//     // create the account -> mint_token
//     invoke_signed(
//         &create_account(
//             signer.key,
//             token_mint.key,
//             lamports,
//             size as u64,
//             token_program.key,
//         ),
//         &[signer.clone(), token_mint.clone()],
//         &[&[TOKEN_MINT_SEED.as_bytes(), &[mint_bump]]],
//     )?;

//     // create the token hook relation
//     invoke(
//         &spl_token_2022::extension::transfer_hook::instruction::initialize(
//             token_program.key,
//             token_mint.key,
//             Some(authority.key.clone()),
//             Some(program_id.clone()),
//         )?,
//         &[token_mint.clone()],
//     )?;

//     // initialize the token_mint
//     let decimals = 9;
//     invoke(
//         &initialize_mint2(
//             token_program.key,
//             token_mint.key,
//             authority.key,
//             Some(authority.key),
//             decimals,
//         )?,
//         &[token_mint.clone()],
//     )?;
//     Ok(())
// }

// // curently working on
// pub fn process_execute(
//     _program_id: &Pubkey,
//     _accounts: &[AccountInfo],
//     _amount: u64,
// ) -> ProgramResult {
//     // source
//     // destination

//     // transfer 1
//     // any source -> program escrow destination
//     //  tax is applied to amount and remains in escrow
//     //  remaining is recorded to the intended receipent

//     //  transfer 2
//     //  program escrow source -> tax vault destination
//     //  must be executed before final transaction
//     //  anyone can exuecte transaction, done from program

//     // transfer 3
//     //  program escrow source -> receipent destenation
//     //  final transaction
//     //  anyone can execute transaction, done from program

//     Ok(())
// }

// // temporary help function -> minting will be down in the process_claim in final version
// pub fn process_mint_tokens(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();

//     let _signer = next_account_info(accounts_iter)?;
//     let receipent = next_account_info(accounts_iter)?;
//     let token_mint = next_account_info(accounts_iter)?;
//     let authority = next_account_info(accounts_iter)?;
//     let token_program = next_account_info(accounts_iter)?;

//     let seeds = &[TOKEN_AUTHORITY_SEED.as_bytes()];
//     let (_account, bump) = Pubkey::find_program_address(seeds, program_id);

//     if authority.key != &_account {
//         return Err(ProgramError::Custom(ErrorCode::InvalidMintAuthority as u32));
//     };

//     // if !token_program.executable {
//     //     return Err(ProgramError::Custom(ErrorCode::AccountNotExecutable as u32));
//     // }

//     // if !token_mint.is_writable {
//     //     return Err(ProgramError::Custom(ErrorCode::Immutable as u32));
//     // }

//     let amount = 1_000_000_000;
//     let decimals = 9;

//     invoke_signed(
//         &mint_to_checked(
//             token_program.key,
//             token_mint.key,
//             receipent.key,
//             authority.key,
//             &[],
//             amount,
//             decimals,
//         )?,
//         &[token_mint.clone(), receipent.clone(), authority.clone()][..],
//         &[&[TOKEN_AUTHORITY_SEED.as_bytes(), &[bump][..]]],
//     )
// }

// pub fn process_transfer_token(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
//     // test cpi transfer from here to see if it works, but I am not sure

//     let accounts_iter = &mut accounts.iter();

//     let authority = next_account_info(accounts_iter)?;
//     let source = next_account_info(accounts_iter)?;
//     let destination = next_account_info(accounts_iter)?;
//     let token_mint = next_account_info(accounts_iter)?;
//     let token_program = next_account_info(accounts_iter)?;

//     let hook_program = next_account_info(accounts_iter)?;
//     let meta_list = next_account_info(accounts_iter)?;

//     let amount = 1_000;
//     let decimals = 9;
//     let mut instruction = transfer_checked(
//         token_program.key,
//         source.key,
//         token_mint.key,
//         destination.key,
//         authority.key,
//         // probably not the best way to handle signer pubkeys, need to dynamically include them if any exist
//         &[],
//         amount,
//         decimals,
//     )?;

//     instruction
//         .accounts
//         .push(AccountMeta::new_readonly(hook_program.key.clone(), false));
//     instruction
//         .accounts
//         .push(AccountMeta::new_readonly(meta_list.key.clone(), false));

//     let account_infos = &[
//         source.clone(),
//         token_mint.clone(),
//         destination.clone(),
//         authority.clone(),
//         hook_program.clone(),
//         meta_list.clone(),
//     ];

//     msg!("{:?}", instruction);

//     invoke(&instruction, account_infos)
// }

// pub fn process_initialize_extra_account_meta_list(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
// ) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();

//     let signer = next_account_info(accounts_iter)?;
//     let token_mint = next_account_info(accounts_iter)?;
//     let extra_account_meta_list = next_account_info(accounts_iter)?;
//     let _system_program = next_account_info(accounts_iter);

//     let account_metas = vec![];
//     let account_size = ExtraAccountMetaList::size_of(account_metas.len())? as u64;
//     let lamports = (Rent::get()?).minimum_balance(account_size as usize);

//     let (_account, bump) = Pubkey::find_program_address(
//         &[b"extra-account-metas", token_mint.key.as_ref()],
//         program_id,
//     );

//     invoke_signed(
//         &create_account(
//             signer.key,
//             extra_account_meta_list.key,
//             lamports,
//             account_size as u64,
//             program_id,
//         ),
//         &[signer.clone(), extra_account_meta_list.clone()],
//         &[&[b"extra-account-metas", token_mint.key.as_ref(), &[bump][..]]],
//     )?;

//     ExtraAccountMetaList::init::<ExecuteInstruction>(
//         &mut extra_account_meta_list.try_borrow_mut_data()?,
//         &account_metas,
//     )?;

//     Ok(())
// }
