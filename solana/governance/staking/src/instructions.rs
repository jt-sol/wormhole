use crate::{
    accounts::{
        ConfigAccount,
        StakeAccountCustodySigner,
        StakePoolCustodySigner,
        StakePoolDeactivatingTokenAccount,
        StakePoolDeactivatingTokenAccountDerivationData,
        StakePoolDistributionTokenAccount,
        StakePoolDistributionTokenAccountDerivationData,
        StakePoolStakingTokenAccount,
        StakePoolStakingTokenAccountDerivationData,
    },
    api::{
        bond_stake_account::BondStakeAccountData,
        close_stake_account::CloseStakeAccountData,
        complete_unbond_stake_account::CompleteUnbondStakeAccountData,
        create_stake_account::{
            CreateStakeAccountData,
            CreateStakeAccountType,
        },
        create_stake_pool::CreateStakePoolData,
        deactivate_stake_pool::DeactivateStakePoolData,
        edit_stake_pool::EditStakePoolData,
        init::InitializeData,
        sync_stake_pool::SyncStakePoolData,
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

pub fn initialize(
    program_id: Pubkey,
    payer: Pubkey,
    unbonding_time: u64,
    staking_token: Pubkey,
) -> solitaire::Result<Instruction> {
    let config_key = ConfigAccount::<'_, { AccountState::Uninitialized }>::key(None, &program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(config_key, false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::Initialize,
            InitializeData {
                unbonding_time,
                staking_token,
            },
        )
            .try_to_vec()?,
    })
}

pub fn bond_stake_account(
    program_id: Pubkey,
    owner: Pubkey,
    stake_account: Pubkey,
    stake_account_token_account: Pubkey,
    stake_pool: Pubkey,
) -> solitaire::Result<Instruction> {
    let config_key = ConfigAccount::<'_, { AccountState::Uninitialized }>::key(None, &program_id);
    let stake_custody_signer = StakeAccountCustodySigner::key(None, &program_id);
    let stake_pool_staking_token_account =
        StakePoolStakingTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolStakingTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(config_key, false),
            AccountMeta::new(stake_account, false),
            AccountMeta::new(stake_account_token_account, false),
            AccountMeta::new_readonly(stake_custody_signer, false),
            AccountMeta::new(stake_pool, false),
            AccountMeta::new(stake_pool_staking_token_account, false),
            AccountMeta::new_readonly(owner, true),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::BondStakeAccount,
            BondStakeAccountData {},
        )
            .try_to_vec()?,
    })
}

pub fn close_stake_account(
    program_id: Pubkey,
    owner: Pubkey,
    stake_account: Pubkey,
    stake_account_token_account: Pubkey,
) -> solitaire::Result<Instruction> {
    let stake_custody_signer = StakeAccountCustodySigner::key(None, &program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(stake_account, false),
            AccountMeta::new(stake_account_token_account, false),
            AccountMeta::new_readonly(stake_custody_signer, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::CloseStakeAccount,
            CloseStakeAccountData {},
        )
            .try_to_vec()?,
    })
}

pub fn complete_unbond_stake_account(
    program_id: Pubkey,
    owner: Pubkey,
    stake_account: Pubkey,
    stake_account_token_account: Pubkey,
    stake_pool: Pubkey,
) -> solitaire::Result<Instruction> {
    let config_key = ConfigAccount::<'_, { AccountState::Uninitialized }>::key(None, &program_id);
    let stake_pool_deactivating_token_account =
        StakePoolDeactivatingTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolDeactivatingTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_custody_signer = StakePoolCustodySigner::key(None, &program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(config_key, false),
            AccountMeta::new(stake_account, false),
            AccountMeta::new(stake_account_token_account, false),
            AccountMeta::new(stake_pool, false),
            AccountMeta::new(stake_pool_deactivating_token_account, false),
            AccountMeta::new_readonly(stake_pool_custody_signer, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::CompleteUnbondStakeAccount,
            CompleteUnbondStakeAccountData {},
        )
            .try_to_vec()?,
    })
}

pub fn create_stake_account(
    program_id: Pubkey,
    payer: Pubkey,
    owner: Pubkey,
    account_type: CreateStakeAccountType,
    stake_account: Pubkey,
    stake_account_token_account: Pubkey,
) -> solitaire::Result<Instruction> {
    let config_key = ConfigAccount::<'_, { AccountState::Uninitialized }>::key(None, &program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(config_key, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(stake_account, true),
            AccountMeta::new(stake_account_token_account, false),
            AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::CreateStakeAccount,
            CreateStakeAccountData {
                owner,
                account_type,
            },
        )
            .try_to_vec()?,
    })
}

pub fn create_stake_pool(
    program_id: Pubkey,
    payer: Pubkey,
    operator: Pubkey,
    stake_pool: Pubkey,
    reward_account: Pubkey,
    staking_mint: Pubkey,
    name: String,
    description: String,
    icon: String,
    commission: u16,
) -> solitaire::Result<Instruction> {
    let config_key = ConfigAccount::<'_, { AccountState::Uninitialized }>::key(None, &program_id);
    let stake_pool_staking_token_account =
        StakePoolStakingTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolStakingTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_distribution_token_account =
        StakePoolDistributionTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolDistributionTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_deactivating_token_account =
        StakePoolDeactivatingTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolDeactivatingTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_custody_signer = StakePoolCustodySigner::key(None, &program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(config_key, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(stake_pool, true),
            AccountMeta::new(stake_pool_staking_token_account, false),
            AccountMeta::new(stake_pool_distribution_token_account, false),
            AccountMeta::new(stake_pool_deactivating_token_account, false),
            AccountMeta::new_readonly(operator, true),
            AccountMeta::new_readonly(reward_account, false),
            AccountMeta::new_readonly(staking_mint, false),
            AccountMeta::new_readonly(stake_pool_custody_signer, false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::CreateStakePool,
            CreateStakePoolData {
                name,
                description,
                icon,
                commission,
            },
        )
            .try_to_vec()?,
    })
}

pub fn deactivate_stake_pool(
    program_id: Pubkey,
    operator: Pubkey,
    stake_pool: Pubkey,
) -> solitaire::Result<Instruction> {
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(stake_pool, false),
            AccountMeta::new_readonly(operator, true),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::DeactivateStakePool,
            DeactivateStakePoolData {},
        )
            .try_to_vec()?,
    })
}

pub fn edit_stake_pool(
    program_id: Pubkey,
    operator: Pubkey,
    stake_pool: Pubkey,
    name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    commission: Option<u16>,
    new_operator: Option<Pubkey>,
) -> solitaire::Result<Instruction> {
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(stake_pool, false),
            AccountMeta::new_readonly(operator, true),
            AccountMeta::new_readonly(
                if let Some(k) = new_operator {
                    k
                } else {
                    operator
                },
                true,
            ),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::EditStakePool,
            EditStakePoolData {
                name,
                description,
                icon,
                commission,
            },
        )
            .try_to_vec()?,
    })
}

pub fn sync_stake_pool(
    program_id: Pubkey,
    stake_pool: Pubkey,
    reward_account: Pubkey,
) -> solitaire::Result<Instruction> {
    let stake_pool_staking_token_account =
        StakePoolStakingTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolStakingTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_distribution_token_account =
        StakePoolDistributionTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolDistributionTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_custody_signer = StakePoolCustodySigner::key(None, &program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(stake_pool, false),
            AccountMeta::new(stake_pool_staking_token_account, false),
            AccountMeta::new(stake_pool_distribution_token_account, false),
            AccountMeta::new_readonly(stake_pool_custody_signer, false),
            AccountMeta::new(reward_account, false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::SyncStakePool,
            SyncStakePoolData {},
        )
            .try_to_vec()?,
    })
}

pub fn unbond_stake_account(
    program_id: Pubkey,
    owner: Pubkey,
    stake_account: Pubkey,
    stake_account_token_account: Pubkey,
    stake_pool: Pubkey,
) -> solitaire::Result<Instruction> {
    let config_key = ConfigAccount::<'_, { AccountState::Uninitialized }>::key(None, &program_id);
    let stake_pool_staking_token_account =
        StakePoolStakingTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolStakingTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_deactivating_token_account =
        StakePoolDeactivatingTokenAccount::<'_, { AccountState::Initialized }>::key(
            &StakePoolDeactivatingTokenAccountDerivationData { stake_pool },
            &program_id,
        );
    let stake_pool_custody_signer = StakePoolCustodySigner::key(None, &program_id);

    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(config_key, false),
            AccountMeta::new(stake_account, false),
            AccountMeta::new(stake_account_token_account, false),
            AccountMeta::new(stake_pool, false),
            AccountMeta::new(stake_pool_staking_token_account, false),
            AccountMeta::new(stake_pool_deactivating_token_account, false),
            AccountMeta::new_readonly(stake_pool_custody_signer, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::BondStakeAccount,
            BondStakeAccountData {},
        )
            .try_to_vec()?,
    })
}

pub fn withdraw_stake_account(
    program_id: Pubkey,
    owner: Pubkey,
    account_type: CreateStakeAccountType,
    stake_account: Pubkey,
    stake_account_token_account: Pubkey,
    recipient: Pubkey,
) -> solitaire::Result<Instruction> {
    let stake_custody_signer = StakeAccountCustodySigner::key(None, &program_id);
    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(stake_account, false),
            AccountMeta::new(recipient, false),
            AccountMeta::new(stake_account_token_account, false),
            AccountMeta::new_readonly(stake_custody_signer, false),
            AccountMeta::new_readonly(owner, true),
            AccountMeta::new_readonly(solana_program::sysvar::clock::id(), false),
            // Dependencies
            AccountMeta::new(solana_program::sysvar::rent::id(), false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ],
        data: (
            crate::instruction::Instruction::CreateStakeAccount,
            CreateStakeAccountData {
                owner,
                account_type,
            },
        )
            .try_to_vec()?,
    })
}
