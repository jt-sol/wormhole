use crate::{
    accounts::{
        StakePoolAccount,
        StakePoolCustodySigner,
        StakePoolDistributionTokenAccount,
        StakePoolDistributionTokenAccountDerivationData,
        StakePoolStakingTokenAccount,
        StakePoolStakingTokenAccountDerivationData,
    },
    types::SplAccount,
    StakingError::InvalidTokenAccount,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use solitaire::{
    processors::seeded::{
        invoke_seeded,
        Seeded,
    },
    *,
};

#[derive(FromAccounts)]
pub struct SyncStakePool<'b> {
    pub stake_pool: Mut<StakePoolAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_staking_token_account:
        Mut<StakePoolStakingTokenAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_distribution_token_account:
        Mut<StakePoolDistributionTokenAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_custody_signer: StakePoolCustodySigner<'b>,

    pub operator_token_account: Mut<Data<'b, SplAccount, { AccountState::Initialized }>>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct SyncStakePoolData {}

pub fn sync_stake_pool(
    ctx: &ExecutionContext,
    accs: &mut SyncStakePool,
    _data: SyncStakePoolData,
) -> Result<()> {
    accs.stake_pool_staking_token_account.verify_derivation(
        ctx.program_id,
        &StakePoolStakingTokenAccountDerivationData {
            stake_pool: *accs.stake_pool.info().key,
        },
    )?;
    accs.stake_pool_distribution_token_account
        .verify_derivation(
            ctx.program_id,
            &StakePoolDistributionTokenAccountDerivationData {
                stake_pool: *accs.stake_pool.info().key,
            },
        )?;

    if *accs.operator_token_account.info().key != accs.stake_pool.reward_account {
        return Err(InvalidTokenAccount.into());
    }

    // Calculate reward distribution
    let rewards = accs.stake_pool_distribution_token_account.amount;
    let operator_share = if accs.stake_pool.commission > 0 {
        ((rewards as u128) * 10000u128 / (accs.stake_pool.commission as u128)) as u64
    } else {
        0
    };
    let staker_share = rewards - operator_share;

    // Transfer coins from distribution to stake account
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        accs.stake_pool_distribution_token_account.info().key,
        accs.stake_pool_staking_token_account.info().key,
        accs.stake_pool_custody_signer.key,
        &[],
        staker_share as u64,
    )?;
    invoke_seeded(&transfer_ix, ctx, &accs.stake_pool_custody_signer, None)?;

    // Transfer coins from distribution to operator account
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        accs.stake_pool_distribution_token_account.info().key,
        accs.operator_token_account.info().key,
        accs.stake_pool_custody_signer.key,
        &[],
        staker_share as u64,
    )?;
    invoke_seeded(&transfer_ix, ctx, &accs.stake_pool_custody_signer, None)?;

    Ok(())
}
