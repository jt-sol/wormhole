use crate::{
    accounts::StakePoolAccount,
    StakingError::InvalidOwner,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use solitaire::{
    *,
};

#[derive(FromAccounts)]
pub struct EditStakePool<'b> {
    pub stake_pool: Mut<StakePoolAccount<'b, { AccountState::Uninitialized }>>,

    pub operator: MaybeMut<Signer<Info<'b>>>,
    // This can be set to the old operator in case there should not be a change
    pub new_operator: MaybeMut<Signer<Info<'b>>>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct EditStakePoolData {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub commission: Option<u16>,
}

pub fn edit_stake_pool(
    _ctx: &ExecutionContext,
    accs: &mut EditStakePool,
    data: EditStakePoolData,
) -> Result<()> {
    if accs.stake_pool.operator != *accs.operator.info().key {
        return Err(InvalidOwner.into());
    }

    if let Some(name) = data.name {
        accs.stake_pool.name = name;
    }
    if let Some(description) = data.description {
        accs.stake_pool.description = description;
    }
    if let Some(icon) = data.icon {
        accs.stake_pool.icon = icon;
    }
    if let Some(commission) = data.commission {
        accs.stake_pool.commission = commission;
    }
    if accs.operator.info().key != accs.new_operator.info().key {
        accs.stake_pool.operator = *accs.new_operator.info().key;
    }

    Ok(())
}
