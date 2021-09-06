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
    processors::seeded::invoke_seeded,
    *,
};

#[derive(FromAccounts)]
pub struct CloseStakeAccount<'b> {
    pub stake_account: StakeAccount<'b, { AccountState::Initialized }>,

    pub to: Mut<Data<'b, SplAccount, { AccountState::Initialized }>>,
    pub custody: Mut<StakeAccountCustodyTokenAccount<'b, { AccountState::Initialized }>>,
    pub custody_signer: StakeAccountCustodySigner<'b>,

    pub owner: Mut<Signer<Info<'b>>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CloseStakeAccountData {}

pub fn close_stake_account(
    ctx: &ExecutionContext,
    accs: &mut CloseStakeAccount,
    _data: CloseStakeAccountData,
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

    // Check that the account has fully vested
    if let StakeAccountType::VESTING(v) = accs.stake_account.account_type {
        if v.cliff_date.checked_add(v.vesting_duration).unwrap()
            < (accs.clock.unix_timestamp as u64)
        {
            return Err(NotVested.into());
        }
    }

    // Drain vesting account
    transfer_sol(
        accs.stake_account.info(),
        accs.owner.info(),
        accs.stake_account.info().lamports(),
    )?;

    // Close token account
    let transfer_ix = spl_token::instruction::close_account(
        &spl_token::id(),
        accs.custody.info().key,
        accs.to.info().key,
        accs.custody_signer.key,
        &[],
    )?;
    invoke_seeded(&transfer_ix, ctx, &accs.custody_signer, None)?;

    Ok(())
}

pub fn transfer_sol(payer_account: &Info, recipient_account: &Info, amount: u64) -> Result<()> {
    let mut payer_balance = payer_account.try_borrow_mut_lamports()?;
    **payer_balance = payer_balance.checked_sub(amount).ok_or(InsufficientFunds)?;
    let mut recipient_balance = recipient_account.try_borrow_mut_lamports()?;
    **recipient_balance = recipient_balance
        .checked_add(amount)
        .ok_or(InsufficientFunds)?;

    Ok(())
}
