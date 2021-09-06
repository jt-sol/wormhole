use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use serde::{
    Deserialize,
    Serialize,
};
use solana_program::pubkey::Pubkey;
use solitaire::{
    pack_type,
    processors::seeded::{
        AccountOwner,
        Owned,
    },
};
use spl_token::state::{
    Account,
    Mint,
};

#[derive(Default, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct ConfigData {
    /// Time it takes for stake to unbond
    pub unbonding_time: u64,
    /// Mint of the token that can be staked
    pub staking_token: Pubkey,
}

impl Owned for ConfigData {
    fn owner(&self) -> AccountOwner {
        AccountOwner::This
    }
}

#[derive(Default, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct StakePoolData {
    pub operator: Pubkey,

    // TODO: Borsh should always reserve at least 32 bytes for each
    pub name: String,
    pub description: String,
    pub icon: String,

    // Commission in bps
    pub commission: u16,
    pub reward_account: Pubkey,

    // Share tracking
    pub total_shares: u128,
    pub total_shares_unbonding: u128,

    pub state: StakePoolState,
}

impl Owned for StakePoolData {
    fn owner(&self) -> AccountOwner {
        AccountOwner::This
    }
}

#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
pub enum StakePoolState {
    ACTIVE,
    DEACTIVATED,
}

impl Default for StakePoolState {
    fn default() -> Self {
        StakePoolState::ACTIVE
    }
}

#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
pub enum StakeAccountState {
    BONDED(Pubkey),
    UNBONDING(UnbondingData),
    UNBONDED,
}

impl Default for StakeAccountState {
    fn default() -> Self {
        StakeAccountState::UNBONDED
    }
}

#[derive(
    Default, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq,
)]
pub struct UnbondingData {
    /// Pubkey of the pool the account is staked to
    pub pool: Pubkey,
    /// Time when the unbonding will be completed
    pub unbonding_time: u64,
    /// Shares of the deactivating pool
    pub unbonding_shares: u128,
}

#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
pub enum StakeAccountType {
    TOKEN,
    VESTING(VestingData),
}

impl Default for StakeAccountType {
    fn default() -> Self {
        StakeAccountType::TOKEN
    }
}

#[derive(
    Default, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq,
)]
pub struct VestingData {
    pub initial_balance: u64,
    pub cliff_date: u64,
    pub vesting_duration: u64,
}

#[derive(Default, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct StakeAccountData {
    pub owner: Pubkey,

    pub custody_account: Pubkey,

    pub account_type: StakeAccountType,
    pub account_state: StakeAccountState,
    pub shares: u128,
}

impl Owned for StakeAccountData {
    fn owner(&self) -> AccountOwner {
        AccountOwner::This
    }
}

pack_type!(SplMint, Mint, AccountOwner::Other(spl_token::id()));
pack_type!(SplAccount, Account, AccountOwner::Other(spl_token::id()));
