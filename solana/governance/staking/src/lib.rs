#![allow(incomplete_features)]
#![feature(const_generics)]

use solitaire::{
    solitaire,
    SolitaireError,
};

pub mod accounts;
pub mod api;
pub mod types;

#[cfg(feature = "no-entrypoint")]
pub mod instructions;

#[cfg(feature = "wasm")]
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
extern crate wasm_bindgen;

#[cfg(feature = "wasm")]
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod wasm;

pub enum StakingError {
    InvalidOwner,
    WrongMint,
    NotVested,
    InsufficientFunds,
    TokenAccountHasDelegation,
    InvalidTokenAccountState,
    NotUnbonded,
    NotUnbonding,
    NotBonded,
    InvalidPool,
    StillUnbonding,
    InvalidTokenAccount,
    StakePoolDeactivated,
}

impl From<StakingError> for SolitaireError {
    fn from(t: StakingError) -> SolitaireError {
        SolitaireError::Custom(t as u64)
    }
}

use crate::api::{
    bond_stake_account::*,
    close_stake_account::*,
    complete_unbond_stake_account::*,
    create_stake_account::*,
    create_stake_pool::*,
    deactivate_stake_pool::*,
    edit_stake_pool::*,
    init::*,
    sync_stake_pool::*,
    unbond_stake_account::*,
    withdraw_stake_account::*,
};

solitaire! {
    BondStakeAccount(BondStakeAccountData) => bond_stake_account,
    CloseStakeAccount(CloseStakeAccountData) => close_stake_account,
    CompleteUnbondStakeAccount(CompleteUnbondStakeAccountData) => complete_unbond_stake_account,
    CreateStakeAccount(CreateStakeAccountData) => create_stake_account,
    CreateStakePool(CreateStakePoolData) => create_stake_pool,
    DeactivateStakePool(DeactivateStakePoolData) => deactivate_stake_pool,
    EditStakePool(EditStakePoolData) => edit_stake_pool,
    Initialize(InitializeData) => initialize,
    SyncStakePool(SyncStakePoolData) => sync_stake_pool,
    UnbondStakeAccount(UnbondStakeAccountData) => unbond_stake_account,
    WithdrawStakeAccount(WithdrawStakeAccountData) => withdraw_stake_account,
}
