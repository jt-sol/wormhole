use crate::{
    accounts::{
        ConfigAccount,
        StakeAccount,
        StakeAccountCustodyTokenAccount,
        StakePoolAccount,
        StakePoolCustodySigner,
        StakePoolDeactivatingTokenAccount,
        StakePoolDeactivatingTokenAccountDerivationData,
    },
    types::StakeAccountState,
    StakingError::{
        InvalidOwner,
        InvalidPool,
        NotUnbonding,
        StillUnbonding,
        WrongMint,
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
pub struct CompleteUnbondStakeAccount<'b> {
    pub config: ConfigAccount<'b, { AccountState::Initialized }>,

    pub stake_account: Mut<StakeAccount<'b, { AccountState::Initialized }>>,
    pub stake_custody: Mut<StakeAccountCustodyTokenAccount<'b, { AccountState::Initialized }>>,

    pub stake_pool: Mut<StakePoolAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_deactivating_token_account:
        Mut<StakePoolDeactivatingTokenAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_custody_signer: StakePoolCustodySigner<'b>,

    pub owner: MaybeMut<Signer<Info<'b>>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CompleteUnbondStakeAccountData {}

pub fn complete_unbond_stake_account(
    ctx: &ExecutionContext,
    accs: &mut CompleteUnbondStakeAccount,
    _data: CompleteUnbondStakeAccountData,
) -> Result<()> {
    if accs.stake_account.owner != *accs.owner.key {
        return Err(InvalidOwner.into());
    }
    accs.stake_pool_deactivating_token_account
        .verify_derivation(
            ctx.program_id,
            &StakePoolDeactivatingTokenAccountDerivationData {
                stake_pool: *accs.stake_pool.info().key,
            },
        )?;
    if accs.stake_account.custody_account != *accs.stake_custody.info().key {
        return Err(WrongMint.into());
    }

    if let StakeAccountState::UNBONDING(d) = accs.stake_account.account_state {
        if *accs.stake_pool.info().key != d.pool {
            return Err(InvalidPool.into());
        }
        if accs.clock.unix_timestamp.unsigned_abs() > d.unbonding_time {
            return Err(StillUnbonding.into());
        }

        // Calculate the underlying value of the shares
        let unstake_amount = (d.unbonding_shares
            * (accs.stake_pool_deactivating_token_account.amount as u128)
            / accs.stake_pool.total_shares_unbonding) as u64;

        // Reduce total unbonding shares
        accs.stake_pool.total_shares_unbonding -= d.unbonding_shares;

        // Transfer tokens
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            accs.stake_pool_deactivating_token_account.info().key,
            accs.stake_custody.info().key,
            accs.stake_pool_custody_signer.key,
            &[],
            unstake_amount as u64,
        )?;
        invoke_seeded(&transfer_ix, ctx, &accs.stake_pool_custody_signer, None)?;

        // Update stake account state
        accs.stake_account.account_state = StakeAccountState::UNBONDED;
    } else {
        return Err(NotUnbonding.into());
    }

    Ok(())
}
