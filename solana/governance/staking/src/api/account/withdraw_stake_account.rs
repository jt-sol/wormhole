use crate::{
    accounts::{
        StakeAccount,
        StakeAccountCustodySigner,
        StakeAccountCustodyTokenAccount,
    },
    types::{
        SplAccount,
        StakeAccountState,
        StakeAccountType,
    },
    StakingError::*,
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
pub struct WithdrawStakeAccount<'b> {
    pub stake_account: StakeAccount<'b, { AccountState::Initialized }>,

    pub to: Mut<Data<'b, SplAccount, { AccountState::Initialized }>>,
    pub custody: Mut<StakeAccountCustodyTokenAccount<'b, { AccountState::Initialized }>>,
    pub custody_signer: StakeAccountCustodySigner<'b>,

    pub owner: MaybeMut<Signer<Info<'b>>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct WithdrawStakeAccountData {
    amount: u64,
}

pub fn withdraw_stake_account(
    ctx: &ExecutionContext,
    accs: &mut WithdrawStakeAccount,
    data: WithdrawStakeAccountData,
) -> Result<()> {
    if accs.stake_account.owner != *accs.owner.key {
        return Err(InvalidOwner.into());
    }
    if accs.stake_account.custody_account != *accs.custody.info().key {
        return Err(WrongMint.into());
    }
    // Check that the stake account is in the correct state
    if accs.stake_account.account_state != StakeAccountState::UNBONDED {
        return Err(NotUnbonded.into());
    }

    accs.custody_signer
        .verify_derivation(ctx.program_id, None)?;

    let amount = match accs.stake_account.account_type {
        StakeAccountType::TOKEN => data.amount,
        StakeAccountType::VESTING(vesting) => {
            // Has the cliff passed
            if (accs.clock.unix_timestamp as u64) < vesting.cliff_date {
                return Ok(());
            }

            // Calculate vested amount
            let remaining_amount = accs.custody.amount;
            let initial_amount = vesting.initial_balance;

            let time_passed = (accs.clock.unix_timestamp as u64)
                .checked_sub(vesting.cliff_date)
                .unwrap();
            let completion = (time_passed as f64 / vesting.vesting_duration as f64).min(1f64);

            let locked_amount = (initial_amount as f64 * (1f64 - completion)) as u64;
            let unlocked_amount = remaining_amount.checked_sub(locked_amount).unwrap();
            if data.amount > unlocked_amount {
                return Err(InsufficientFunds.into());
            }
            data.amount.min(unlocked_amount)
        }
    };

    // Transfer withdrawn tokens
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        accs.custody.info().key,
        accs.to.info().key,
        accs.custody_signer.key,
        &[],
        amount,
    )?;
    invoke_seeded(&transfer_ix, ctx, &accs.custody_signer, None)?;

    Ok(())
}
