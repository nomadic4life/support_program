//! Program entrypoint

use crate::processor::{
    // PROCESSOR
    process_execute,
    process_initialize_extra_account_meta_list,
    process_initialize_token_mint,
    process_mint_tokens,
    Instructions,
};
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
use spl_transfer_hook_interface::instruction::TransferHookInstruction;

solana_program::entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let code = u64::from_be_bytes(
        instruction_data[..8]
            .try_into()
            .expect("slice with incorrect length"),
    );

    msg!("CODE: {}, {}", code, instruction_data[8]);

    let (instruction, _data) = match code {
        // Normal Instructions
        0 => (Instructions::try_from_slice(&[instruction_data[8]])?, None),

        // Transfer Hook Instructions
        _ => match TransferHookInstruction::unpack(instruction_data)? {
            // Execute
            TransferHookInstruction::Execute { amount } => (Instructions::Execute { amount }, None),

            // InitializeExtraAccountMetaList
            TransferHookInstruction::InitializeExtraAccountMetaList {
                extra_account_metas,
            } => (
                Instructions::InitializeExtraAccountMetaList,
                Some(extra_account_metas),
            ),

            // UpdateExtraAccountMetaList
            TransferHookInstruction::UpdateExtraAccountMetaList {
                extra_account_metas,
            } => (
                Instructions::UpdateExtraAccountMetaList,
                Some(extra_account_metas),
            ),
        },
    };

    msg!("instruction: {:?}", instruction);

    match instruction {
        Instructions::InitializeTokenMint => process_initialize_token_mint(program_id, accounts),
        Instructions::MintTokens { amount } => process_mint_tokens(program_id, accounts, amount),
        Instructions::InitializeExtraAccountMetaList => {
            process_initialize_extra_account_meta_list(program_id, accounts)
        }
        Instructions::Execute { amount } => process_execute(program_id, accounts, amount),
        _ => fallback(program_id, accounts),
    }
}

pub fn fallback(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}
