use crate::{
    accounts::{
        ConfigAccount,
        StakeAccount,
        StakePoolAccount,
        StakePoolCustodySigner,
        StakePoolDeactivatingTokenAccount,
        StakePoolDeactivatingTokenAccountDerivationData,
        StakePoolStakingTokenAccount,
        StakePoolStakingTokenAccountDerivationData,
    },
    types::{
        StakeAccountState,
        UnbondingData,
    },
    StakingError::{
        InvalidOwner,
        InvalidPool,
        NotBonded,
    },
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::sysvar::clock::Clock;
use solitaire::{
    processors::seeded::{
        invoke_seeded,
        Seeded,
    },
    *,
};

#[derive(FromAccounts)]
pub struct UnbondStakeAccount<'b> {
    pub config: ConfigAccount<'b, { AccountState::Initialized }>,

    pub stake_account: Mut<StakeAccount<'b, { AccountState::Initialized }>>,

    pub stake_pool: Mut<StakePoolAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_staking_token_account:
        Mut<StakePoolStakingTokenAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_deactivating_token_account:
        Mut<StakePoolDeactivatingTokenAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_custody_signer: StakePoolCustodySigner<'b>,

    pub owner: MaybeMut<Signer<Info<'b>>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct UnbondStakeAccountData {}

pub fn unbond_stake_account(
    ctx: &ExecutionContext,
    accs: &mut UnbondStakeAccount,
    _data: UnbondStakeAccountData,
) -> Result<()> {
    if accs.stake_account.owner != *accs.owner.key {
        return Err(InvalidOwner.into());
    }
    accs.stake_pool_staking_token_account.verify_derivation(
        ctx.program_id,
        &StakePoolStakingTokenAccountDerivationData {
            stake_pool: *accs.stake_pool.info().key,
        },
    )?;
    accs.stake_pool_deactivating_token_account
        .verify_derivation(
            ctx.program_id,
            &StakePoolDeactivatingTokenAccountDerivationData {
                stake_pool: *accs.stake_pool.info().key,
            },
        )?;

    if let StakeAccountState::BONDED(pool) = accs.stake_account.account_state {
        if *accs.stake_pool.info().key != pool {
            return Err(InvalidPool.into());
        }
    } else {
        return Err(NotBonded.into());
    }

    // Reduce current total shares by withdrawn shares
    accs.stake_pool.total_shares -= accs.stake_account.shares;

    // Calculate the underlying value of the shares
    let unstake_amount = (accs.stake_account.shares
        * (accs.stake_pool_staking_token_account.amount as u128)
        / accs.stake_pool.total_shares) as u64;
    // Calculate shares in the deactivating pool (this must be done before the transfer)
    let shares = if accs.stake_pool.total_shares_unbonding == 0 {
        unstake_amount as u128
    } else {
        ((unstake_amount as u128) * accs.stake_pool.total_shares_unbonding)
            / accs.stake_pool_deactivating_token_account.amount as u128
    };

    // Add new unbonding shares to the total pool
    accs.stake_pool.total_shares_unbonding += shares;

    // Transfer tokens
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        accs.stake_pool_staking_token_account.info().key,
        accs.stake_pool_deactivating_token_account.info().key,
        accs.stake_pool_custody_signer.key,
        &[],
        unstake_amount,
    )?;
    invoke_seeded(&transfer_ix, ctx, &accs.stake_pool_custody_signer, None)?;

    // Update stake account state
    accs.stake_account.shares = 0;
    accs.stake_account.account_state = StakeAccountState::UNBONDING(UnbondingData {
        pool: *accs.stake_pool.info().key,
        unbonding_time: accs.clock.unix_timestamp.unsigned_abs() + accs.config.unbonding_time,
        unbonding_shares: shares,
    });

    Ok(())
}
