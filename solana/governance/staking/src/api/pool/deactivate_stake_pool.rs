use crate::{
    accounts::StakePoolAccount,
    types::StakePoolState,
    StakingError::{
        InvalidOwner,
        StakePoolDeactivated,
    },
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use solitaire::{
    *,
};

#[derive(FromAccounts)]
pub struct DeactivateStakePool<'b> {
    pub stake_pool: Mut<StakePoolAccount<'b, { AccountState::Uninitialized }>>,

    pub operator: MaybeMut<Signer<Info<'b>>>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct DeactivateStakePoolData {}

pub fn deactivate_stake_pool(
    _ctx: &ExecutionContext,
    accs: &mut DeactivateStakePool,
    _data: DeactivateStakePoolData,
) -> Result<()> {
    if accs.stake_pool.operator != *accs.operator.info().key {
        return Err(InvalidOwner.into());
    }
    if accs.stake_pool.state != StakePoolState::ACTIVE {
        return Err(StakePoolDeactivated.into());
    }

    // Set the stake pool to the deactivated state.
    accs.stake_pool.state = StakePoolState::DEACTIVATED;

    Ok(())
}
