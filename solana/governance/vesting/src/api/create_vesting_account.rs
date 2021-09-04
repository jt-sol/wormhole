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
        InvalidTokenAccountState,
        TokenAccountHasDelegation,
    },
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::{
    program_option::COption,
    pubkey::Pubkey,
    sysvar::clock::Clock,
};
use solitaire::{
    processors::seeded::Seeded,
    CreationLamports::Exempt,
    *,
};

#[derive(FromAccounts)]
pub struct CreateVestingAccount<'b> {
    pub payer: Mut<Signer<Info<'b>>>,

    pub vesting_account: Mut<Signer<VestingAccount<'b, { AccountState::Uninitialized }>>>,
    pub custody: Mut<CustodyTokenAccount<'b, { AccountState::Initialized }>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CreateVestingAccountData {
    pub owner: Pubkey,
    pub cliff_date: u64,
    pub vesting_duration: u64,
}

pub fn create_vesting_account(
    ctx: &ExecutionContext,
    accs: &mut CreateVestingAccount,
    data: CreateVestingAccountData,
) -> Result<()> {
    // Custody account must be owned by vesting program
    if accs.custody.owner != VestingCustodySigner::key(None, ctx.program_id) {
        return Err(InvalidOwner.into());
    }
    // Custody account must not have tokens delegated
    if accs.custody.delegate != COption::None {
        return Err(TokenAccountHasDelegation.into());
    }
    // Custody account must only be closable by vesting program
    if accs.custody.close_authority != COption::None {
        return Err(TokenAccountHasDelegation.into());
    }
    // Token account must not be frozen
    if accs.custody.state != spl_token::state::AccountState::Initialized {
        return Err(InvalidTokenAccountState.into());
    }

    // Set fields on vesting account
    accs.vesting_account.owner = data.owner;
    accs.vesting_account.cliff_date = data.cliff_date;
    accs.vesting_account.vesting_duration = data.vesting_duration;
    accs.vesting_account.creation_date = accs.clock.unix_timestamp.unsigned_abs();
    accs.vesting_account.amount = accs.custody.amount;

    // Create vesting account
    let size = accs.vesting_account.size();
    let ix = solana_program::system_instruction::create_account(
        accs.payer.key,
        accs.vesting_account.info().key,
        Exempt.amount(size),
        size as u64,
        ctx.program_id,
    );
    solana_program::program::invoke(&ix, ctx.accounts)?;

    Ok(())
}
