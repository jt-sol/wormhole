use crate::{
    accounts::{
        ConfigAccount,
        StakeAccount,
        StakeAccountCustodySigner,
        StakeAccountCustodyTokenAccount,
    },
    types::{
        StakeAccountState,
        StakeAccountType,
        VestingData,
    },
    StakingError::*,
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
pub struct CreateStakeAccount<'b> {
    pub config: ConfigAccount<'b, { AccountState::Initialized }>,
    pub payer: Mut<Signer<Info<'b>>>,

    pub stake_account: Mut<Signer<StakeAccount<'b, { AccountState::Uninitialized }>>>,
    pub custody: Mut<StakeAccountCustodyTokenAccount<'b, { AccountState::Initialized }>>,

    pub clock: Sysvar<'b, Clock>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CreateStakeAccountData {
    pub owner: Pubkey,
    pub account_type: CreateStakeAccountType,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum CreateStakeAccountType {
    NORMAL,
    VESTING(CreateStakeAccountVestingData),
}

impl Default for CreateStakeAccountType {
    fn default() -> Self {
        CreateStakeAccountType::NORMAL
    }
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CreateStakeAccountVestingData {
    pub cliff_date: u64,
    pub vesting_duration: u64,
}

pub fn create_stake_account(
    ctx: &ExecutionContext,
    accs: &mut CreateStakeAccount,
    data: CreateStakeAccountData,
) -> Result<()> {
    // Custody account must be owned by vesting program
    if accs.custody.owner != StakeAccountCustodySigner::key(None, ctx.program_id) {
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
    // Token account must be of the staking token
    if accs.config.staking_token != accs.custody.mint {
        return Err(WrongMint.into());
    }

    // Set fields on staking account
    accs.stake_account.owner = data.owner;
    accs.stake_account.account_state = StakeAccountState::UNBONDED;
    accs.stake_account.account_type = match data.account_type {
        CreateStakeAccountType::NORMAL => StakeAccountType::TOKEN,
        CreateStakeAccountType::VESTING(v) => StakeAccountType::VESTING(VestingData {
            initial_balance: accs.custody.amount,
            cliff_date: v.cliff_date,
            vesting_duration: v.vesting_duration,
        }),
    };
    accs.stake_account.custody_account = *accs.custody.info().key;

    // Create staking account
    let size = accs.stake_account.size();
    let ix = solana_program::system_instruction::create_account(
        accs.payer.key,
        accs.stake_account.info().key,
        Exempt.amount(size),
        size as u64,
        ctx.program_id,
    );
    solana_program::program::invoke(&ix, ctx.accounts)?;

    Ok(())
}
