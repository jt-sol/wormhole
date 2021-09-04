#![feature(const_generics)]

use api::{
    claim_tokens::*,
    close_vesting_account::*,
    create_vesting_account::*,
    transfer_ownership::*,
};
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

pub enum VestingError {
    InvalidOwner,
    WrongMint,
    NotVested,
    InsufficientFunds,
    TokenAccountHasDelegation,
    InvalidTokenAccountState,
}

impl From<VestingError> for SolitaireError {
    fn from(t: VestingError) -> SolitaireError {
        SolitaireError::Custom(t as u64)
    }
}

solitaire! {
    CreateVestingAccount(CreateVestingAccountData) => create_vesting_account,
    ClaimTokens(ClaimTokensData) => claim_tokens,
    CloseVestingAccount(CloseVestingAccountData) => close_vesting_account,
    TransferOwnership(TransferOwnershipData) => transfer_ownership,
}
