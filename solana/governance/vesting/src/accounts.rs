use crate::types::{
    SplAccount,
    VestingAccountData,
};
use solana_program::pubkey::Pubkey;
use solitaire::{
    processors::seeded::Seeded,
    AccountState,
    Data,
    Derive,
    Info,
};

pub type CustodyTokenAccount<'a, const State: AccountState> = Data<'a, SplAccount, { State }>;

pub type VestingAccount<'a, const State: AccountState> = Data<'a, VestingAccountData, { State }>;

pub type VestingCustodySigner<'a> = Derive<Info<'a>, "custody">;
