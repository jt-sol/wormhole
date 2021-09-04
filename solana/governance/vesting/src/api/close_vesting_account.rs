use crate::{
    accounts::{
        CustodyTokenAccount,
        VestingAccount,
        VestingCustodySigner,
    },
    types::SplAccount,
    VestingError::{
        InsufficientFunds,
        InvalidOwner,
        NotVested,
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
pub struct CloseVestingAccount<'b> {
    pub vesting_account: VestingAccount<'b, { AccountState::Initialized }>,

    pub to: Mut<Data<'b, SplAccount, { AccountState::Initialized }>>,
    pub custody: Mut<CustodyTokenAccount<'b, { AccountState::Initialized }>>,
    pub custody_signer: VestingCustodySigner<'b>,

    pub owner: Mut<Signer<Info<'b>>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CloseVestingAccountData {}

pub fn close_vesting_account(
    ctx: &ExecutionContext,
    accs: &mut CloseVestingAccount,
    _data: CloseVestingAccountData,
) -> Result<()> {
    if accs.vesting_account.owner != *accs.owner.key {
        return Err(InvalidOwner.into());
    }
    if accs.vesting_account.token_account != *accs.custody.info().key {
        return Err(WrongMint.into());
    }

    // Check that the account has fully vested
    if accs
        .vesting_account
        .cliff_date
        .checked_add(accs.vesting_account.vesting_duration)
        .unwrap()
        < (accs.clock.unix_timestamp as u64)
    {
        return Err(NotVested.into());
    }

    // Drain vesting account
    transfer_sol(
        accs.vesting_account.info(),
        accs.owner.info(),
        accs.vesting_account.info().lamports(),
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
