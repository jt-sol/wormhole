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
pub struct VestingAccountData {
    pub owner: Pubkey,

    pub amount: u64,
    pub token_account: Pubkey,

    pub creation_date: u64,
    pub cliff_date: u64,
    pub vesting_duration: u64,
}

impl Owned for VestingAccountData {
    fn owner(&self) -> AccountOwner {
        AccountOwner::This
    }
}

pack_type!(SplMint, Mint, AccountOwner::Other(spl_token::id()));
pack_type!(SplAccount, Account, AccountOwner::Other(spl_token::id()));
