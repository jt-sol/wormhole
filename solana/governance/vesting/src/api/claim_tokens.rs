use crate::{
    accounts::{
        CustodyTokenAccount,
        VestingAccount,
        VestingCustodySigner,
    },
    types::{
        SplAccount,
        SplMint,
    },
    VestingError::{
        InvalidOwner,
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
pub struct ClaimTokens<'b> {
    pub vesting_account: VestingAccount<'b, { AccountState::Initialized }>,

    pub to: Mut<Data<'b, SplAccount, { AccountState::Initialized }>>,
    pub custody: Mut<CustodyTokenAccount<'b, { AccountState::Initialized }>>,
    pub custody_signer: VestingCustodySigner<'b>,

    pub owner: MaybeMut<Signer<Info<'b>>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct ClaimTokensData {}

pub fn claim_tokens(
    ctx: &ExecutionContext,
    accs: &mut ClaimTokens,
    _data: ClaimTokensData,
) -> Result<()> {
    if accs.vesting_account.owner != *accs.owner.key {
        return Err(InvalidOwner.into());
    }
    if accs.vesting_account.token_account != *accs.custody.info().key {
        return Err(WrongMint.into());
    }

    accs.custody_signer
        .verify_derivation(ctx.program_id, None)?;

    // Has the cliff passed
    if (accs.clock.unix_timestamp as u64) < accs.vesting_account.cliff_date {
        return Ok(());
    }

    // Calculate vested amount
    let remaining_amount = accs.custody.amount;
    let initial_amount = accs.vesting_account.amount;
    let already_claimed = initial_amount.checked_sub(remaining_amount).unwrap();

    let time_passed = (accs.clock.unix_timestamp as u64)
        .checked_sub(accs.vesting_account.cliff_date)
        .unwrap();
    let completion = (time_passed as f64 / accs.vesting_account.vesting_duration as f64).min(1f64);

    let unlocked_amount = (initial_amount as f64 * completion) as u64;
    let amount_to_unlock = unlocked_amount.checked_sub(already_claimed).unwrap();

    // Transfer vested tokens
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        accs.custody.info().key,
        accs.to.info().key,
        accs.custody_signer.key,
        &[],
        amount_to_unlock,
    )?;
    invoke_seeded(&transfer_ix, ctx, &accs.custody_signer, None)?;

    Ok(())
}
