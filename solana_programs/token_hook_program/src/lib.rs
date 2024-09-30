// Instructions
//  TransferHookInstruction::Execute
//  TransferHookInstruction::InitializeExtraAccountMetaList
//  TransferHookInstruction::UpdateExtraAccountMetaList
//  ::InitializeTokenMint
//  ::MintTo

pub mod instruction;
mod processor;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
