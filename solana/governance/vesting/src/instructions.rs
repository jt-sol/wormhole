use crate::{
    accounts::VestingCustodySigner,
    api::{
        claim_tokens::ClaimTokensData,
        close_vesting_account::CloseVestingAccountData,
        create_vesting_account::CreateVestingAccountData,
        transfer_ownership::TransferOwnershipData,
    },
};
use borsh::BorshSerialize;
use solana_program::{
    instruction::{
        AccountMeta,
        Instruction,
    },
    pubkey::Pubkey,
};
use solitaire::{
    processors::seeded::Seeded,
    AccountState,
};
use spl_token::state::Mint;
use std::str::FromStr;

pub fn create_vesting_account(
    program_id: Pubkey,
    payer: Pubkey,
    vesting_account: Pubkey,
    custody_token_account: Pubkey,
    owner: Pubkey,
    cliff_date: u64,
    vesting_duration: u64,
) -> solitaire::Result<Instruction> {
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(vesting_account, true),
            AccountMeta::new(custody_token_account, false),
            AccountMeta::new(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::CreateVestingAccount,
            CreateVestingAccountData {
                owner,
                cliff_date,
                vesting_duration,
            },
        )
            .try_to_vec()?,
    })
}

pub fn close_vesting_account(
    program_id: Pubkey,
    vesting_account: Pubkey,
    custody_token_account: Pubkey,
    rest_recipient: Pubkey,
    owner: Pubkey,
) -> solitaire::Result<Instruction> {
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vesting_account, false),
            AccountMeta::new(rest_recipient, false),
            AccountMeta::new(custody_token_account, false),
            AccountMeta::new_readonly(VestingCustodySigner::key(None, &program_id), false),
            AccountMeta::new(owner, true),
            AccountMeta::new(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::CloseVestingAccount,
            CloseVestingAccountData {},
        )
            .try_to_vec()?,
    })
}

pub fn claim_tokens(
    program_id: Pubkey,
    vesting_account: Pubkey,
    custody_token_account: Pubkey,
    recipient: Pubkey,
    owner: Pubkey,
) -> solitaire::Result<Instruction> {
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vesting_account, false),
            AccountMeta::new(recipient, false),
            AccountMeta::new(custody_token_account, false),
            AccountMeta::new_readonly(VestingCustodySigner::key(None, &program_id), false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::ClaimTokens,
            ClaimTokensData {},
        )
            .try_to_vec()?,
    })
}

pub fn transfer_ownership(
    program_id: Pubkey,
    vesting_account: Pubkey,
    owner: Pubkey,
    new_owner: Pubkey,
) -> solitaire::Result<Instruction> {
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vesting_account, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(new_owner, true),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::TransferOwnership,
            TransferOwnershipData {},
        )
            .try_to_vec()?,
    })
}
