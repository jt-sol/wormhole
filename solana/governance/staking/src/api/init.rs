use crate::accounts::ConfigAccount;
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use solana_program::pubkey::Pubkey;
use solitaire::{
    CreationLamports::Exempt,
    *,
};

#[derive(FromAccounts)]
pub struct Initialize<'b> {
    pub payer: Mut<Signer<Info<'b>>>,

    pub config: Mut<ConfigAccount<'b, { AccountState::Uninitialized }>>,
}

#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct InitializeData {
    pub unbonding_time: u64,
    pub staking_token: Pubkey,
}

pub fn initialize(
    ctx: &ExecutionContext,
    accs: &mut Initialize,
    data: InitializeData,
) -> Result<()> {
    // Create the config account
    accs.config.create(ctx, accs.payer.key, Exempt)?;
    accs.config.unbonding_time = data.unbonding_time;
    accs.config.staking_token = data.staking_token;
    Ok(())
}
