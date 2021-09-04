use crate::{
    accounts::VestingAccount,
    VestingError::InvalidOwner,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::sysvar::clock::Clock;
use solitaire::*;

#[derive(FromAccounts)]
pub struct TransferOwnership<'b> {
    pub vesting_account: Mut<VestingAccount<'b, { AccountState::Initialized }>>,

    pub owner: MaybeMut<Signer<Info<'b>>>,
    pub new_owner: Signer<Info<'b>>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct TransferOwnershipData {}

pub fn transfer_ownership(
    _ctx: &ExecutionContext,
    accs: &mut TransferOwnership,
    _data: TransferOwnershipData,
) -> Result<()> {
    if accs.vesting_account.owner != *accs.owner.key {
        return Err(InvalidOwner.into());
    }

    accs.vesting_account.owner = *accs.new_owner.info().key;

    Ok(())
}
