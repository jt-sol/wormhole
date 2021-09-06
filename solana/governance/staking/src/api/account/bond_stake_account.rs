use crate::{
    accounts::{
        ConfigAccount,
        StakeAccount,
        StakeAccountCustodySigner,
        StakeAccountCustodyTokenAccount,
        StakePoolAccount,
        StakePoolStakingTokenAccount,
        StakePoolStakingTokenAccountDerivationData,
    },
    types::{
        StakeAccountState,
        StakePoolState,
    },
    StakingError::{
        InvalidOwner,
        NotUnbonded,
        StakePoolDeactivated,
        WrongMint,
    },
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use solitaire::{
    processors::seeded::{
        invoke_seeded,
        Seeded,
    },
    *,
};

#[derive(FromAccounts)]
pub struct BondStakeAccount<'b> {
    pub config: ConfigAccount<'b, { AccountState::Initialized }>,

    pub stake_account: Mut<StakeAccount<'b, { AccountState::Initialized }>>,
    pub stake_custody: Mut<StakeAccountCustodyTokenAccount<'b, { AccountState::Initialized }>>,
    pub stake_custody_signer: StakeAccountCustodySigner<'b>,

    pub stake_pool: Mut<StakePoolAccount<'b, { AccountState::Initialized }>>,
    pub stake_pool_staking_token_account:
        Mut<StakePoolStakingTokenAccount<'b, { AccountState::Initialized }>>,

    pub owner: MaybeMut<Signer<Info<'b>>>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct BondStakeAccountData {}

pub fn bond_stake_account(
    ctx: &ExecutionContext,
    accs: &mut BondStakeAccount,
    _data: BondStakeAccountData,
) -> Result<()> {
    if accs.stake_account.owner != *accs.owner.key {
        return Err(InvalidOwner.into());
    }
    if accs.stake_account.custody_account != *accs.stake_custody.info().key {
        return Err(WrongMint.into());
    }
    if accs.stake_account.account_state != StakeAccountState::UNBONDED {
        return Err(NotUnbonded.into());
    }
    accs.stake_pool_staking_token_account.verify_derivation(
        ctx.program_id,
        &StakePoolStakingTokenAccountDerivationData {
            stake_pool: *accs.stake_pool.info().key,
        },
    )?;
    if accs.stake_pool.state == StakePoolState::DEACTIVATED {
        return Err(StakePoolDeactivated.into());
    }

    // Calculate shares (this must be done before the transfer)
    let stake_amount = accs.stake_custody.amount;
    let shares = if accs.stake_pool.total_shares == 0 {
        stake_amount as u128
    } else {
        ((stake_amount as u128) * accs.stake_pool.total_shares)
            / accs.stake_pool_staking_token_account.amount as u128
    };

    // Track shares and staking state
    accs.stake_pool.total_shares += shares;
    accs.stake_account.shares = shares;
    accs.stake_account.account_state = StakeAccountState::BONDED(*accs.stake_pool.info().key);

    // Transfer tokens
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        accs.stake_custody.info().key,
        accs.stake_pool_staking_token_account.info().key,
        accs.stake_custody_signer.key,
        &[],
        accs.stake_custody.amount,
    )?;
    invoke_seeded(&transfer_ix, ctx, &accs.stake_custody_signer, None)?;

    Ok(())
}
