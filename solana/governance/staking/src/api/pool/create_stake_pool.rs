use crate::{
    accounts::{
        ConfigAccount,
        StakePoolAccount,
        StakePoolCustodySigner,
        StakePoolDeactivatingTokenAccount,
        StakePoolDeactivatingTokenAccountDerivationData,
        StakePoolDistributionTokenAccount,
        StakePoolDistributionTokenAccountDerivationData,
        StakePoolStakingTokenAccount,
        StakePoolStakingTokenAccountDerivationData,
    },
    types::{
        SplAccount,
        SplMint,
    },
    StakingError::*,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::program::invoke_signed;
use solitaire::{
    CreationLamports::Exempt,
    *,
};

#[derive(FromAccounts)]
pub struct CreateStakePool<'b> {
    pub config: ConfigAccount<'b, { AccountState::Initialized }>,
    pub payer: Mut<Signer<Info<'b>>>,

    pub stake_pool: Mut<Signer<StakePoolAccount<'b, { AccountState::Uninitialized }>>>,
    pub staking_token_account:
        Mut<StakePoolStakingTokenAccount<'b, { AccountState::Uninitialized }>>,
    pub distribution_token_account:
        Mut<StakePoolDistributionTokenAccount<'b, { AccountState::Uninitialized }>>,
    pub deactivating_token_account:
        Mut<StakePoolDeactivatingTokenAccount<'b, { AccountState::Uninitialized }>>,

    pub operator: MaybeMut<Signer<Info<'b>>>,
    pub reward_account: Data<'b, SplAccount, { AccountState::Initialized }>,
    pub staking_mint: Data<'b, SplMint, { AccountState::Initialized }>,
    pub custody_signer: StakePoolCustodySigner<'b>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CreateStakePoolData {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub commission: u16,
}

pub fn create_stake_pool(
    ctx: &ExecutionContext,
    accs: &mut CreateStakePool,
    data: CreateStakePoolData,
) -> Result<()> {
    if accs.config.staking_token != *accs.staking_mint.info().key {
        return Err(WrongMint.into());
    }
    if accs.config.staking_token != accs.reward_account.mint {
        return Err(WrongMint.into());
    }

    // Create staking token account (derivation implicitly verified here)
    accs.staking_token_account.create(
        &StakePoolStakingTokenAccountDerivationData {
            stake_pool: *accs.stake_pool.info().key,
        },
        ctx,
        accs.payer.key,
        Exempt,
    )?;

    let init_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        accs.staking_token_account.info().key,
        accs.staking_mint.info().key,
        accs.custody_signer.key,
    )?;
    invoke_signed(&init_ix, ctx.accounts, &[])?;

    // Create distribution token account (derivation implicitly verified here)
    accs.distribution_token_account.create(
        &StakePoolDistributionTokenAccountDerivationData {
            stake_pool: *accs.stake_pool.info().key,
        },
        ctx,
        accs.payer.key,
        Exempt,
    )?;

    let init_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        accs.distribution_token_account.info().key,
        accs.staking_mint.info().key,
        accs.custody_signer.key,
    )?;
    invoke_signed(&init_ix, ctx.accounts, &[])?;

    // Create deactivating token account (derivation implicitly verified here)
    accs.deactivating_token_account.create(
        &StakePoolDeactivatingTokenAccountDerivationData {
            stake_pool: *accs.stake_pool.info().key,
        },
        ctx,
        accs.payer.key,
        Exempt,
    )?;

    let init_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        accs.deactivating_token_account.info().key,
        accs.staking_mint.info().key,
        accs.custody_signer.key,
    )?;
    invoke_signed(&init_ix, ctx.accounts, &[])?;

    // Populate stake pool fields
    accs.stake_pool.name = data.name;
    accs.stake_pool.description = data.description;
    accs.stake_pool.icon = data.icon;
    accs.stake_pool.commission = data.commission;
    accs.stake_pool.reward_account = *accs.reward_account.info().key;
    accs.stake_pool.operator = *accs.operator.info().key;

    // Create stake pool
    let size = accs.stake_pool.size();
    let ix = solana_program::system_instruction::create_account(
        accs.payer.key,
        accs.stake_pool.info().key,
        Exempt.amount(size),
        size as u64,
        ctx.program_id,
    );
    solana_program::program::invoke(&ix, ctx.accounts)?;

    Ok(())
}
