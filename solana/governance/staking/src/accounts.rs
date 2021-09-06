use crate::types::{
    ConfigData,
    SplAccount,
    StakeAccountData,
    StakePoolData,
};
use solana_program::pubkey::Pubkey;
use solitaire::{
    processors::seeded::Seeded,
    AccountState,
    Data,
    Derive,
    Info,
};

pub type ConfigAccount<'a, const STATE: AccountState> =
    Derive<Data<'a, ConfigData, { STATE }>, "config">;

pub type StakeAccountCustodyAccount<'a, const STATE: AccountState> =
    Data<'a, SplAccount, { STATE }>;

pub type StakeAccount<'a, const STATE: AccountState> = Data<'a, StakeAccountData, { STATE }>;
pub type StakeAccountCustodyTokenAccount<'a, const STATE: AccountState> =
    Data<'a, SplAccount, { STATE }>;

pub type StakePoolAccount<'a, const STATE: AccountState> = Data<'a, StakePoolData, { STATE }>;
pub type StakePoolDistributionTokenAccount<'a, const STATE: AccountState> =
    Data<'a, SplAccount, { STATE }>;

pub struct StakePoolDistributionTokenAccountDerivationData {
    pub stake_pool: Pubkey,
}

impl<'b, const STATE: AccountState> Seeded<&StakePoolDistributionTokenAccountDerivationData>
    for StakePoolDistributionTokenAccount<'b, { STATE }>
{
    fn seeds(accs: &StakePoolDistributionTokenAccountDerivationData) -> Vec<Vec<u8>> {
        vec![
            String::from("pool_distribution").as_bytes().to_vec(),
            accs.stake_pool.to_bytes().to_vec(),
        ]
    }
}

pub type StakePoolDeactivatingTokenAccount<'a, const STATE: AccountState> =
    Data<'a, SplAccount, { STATE }>;

pub struct StakePoolDeactivatingTokenAccountDerivationData {
    pub stake_pool: Pubkey,
}

impl<'b, const STATE: AccountState> Seeded<&StakePoolDeactivatingTokenAccountDerivationData>
    for StakePoolDeactivatingTokenAccount<'b, { STATE }>
{
    fn seeds(accs: &StakePoolDeactivatingTokenAccountDerivationData) -> Vec<Vec<u8>> {
        vec![
            String::from("pool_deactivating").as_bytes().to_vec(),
            accs.stake_pool.to_bytes().to_vec(),
        ]
    }
}

pub type StakePoolStakingTokenAccount<'a, const STATE: AccountState> =
    Data<'a, SplAccount, { STATE }>;

pub struct StakePoolStakingTokenAccountDerivationData {
    pub stake_pool: Pubkey,
}

impl<'b, const STATE: AccountState> Seeded<&StakePoolStakingTokenAccountDerivationData>
    for StakePoolStakingTokenAccount<'b, { STATE }>
{
    fn seeds(accs: &StakePoolStakingTokenAccountDerivationData) -> Vec<Vec<u8>> {
        vec![
            String::from("pool_staking").as_bytes().to_vec(),
            accs.stake_pool.to_bytes().to_vec(),
        ]
    }
}

pub type StakeAccountCustodySigner<'a> = Derive<Info<'a>, "stake_account_custody">;
pub type StakePoolCustodySigner<'a> = Derive<Info<'a>, "stake_pool_custody">;
