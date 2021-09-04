use crate::{
    accounts::VestingCustodySigner,
    types::VestingAccountData,
};
use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;
use solitaire::{
    processors::seeded::Seeded,
    AccountState,
};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn create_vesting_account(
    program_id: String,
    payer: String,
    vesting_account: String,
    custody_token_account: String,
    owner: String,
    cliff_date: u64,
    vesting_duration: u64,
) -> JsValue {
    let program_id = Pubkey::from_str(program_id.as_str()).unwrap();
    let payer = Pubkey::from_str(payer.as_str()).unwrap();
    let vesting_account = Pubkey::from_str(vesting_account.as_str()).unwrap();
    let custody_token_account = Pubkey::from_str(custody_token_account.as_str()).unwrap();
    let owner = Pubkey::from_str(owner.as_str()).unwrap();

    let ix = crate::instructions::create_vesting_account(
        program_id,
        payer,
        vesting_account,
        custody_token_account,
        owner,
        cliff_date,
        vesting_duration,
    )
    .unwrap();

    JsValue::from_serde(&ix).unwrap()
}

#[wasm_bindgen]
pub fn close_vesting_account(
    program_id: String,
    vesting_account: String,
    custody_token_account: String,
    rest_recipient: String,
    owner: String,
) -> JsValue {
    let program_id = Pubkey::from_str(program_id.as_str()).unwrap();
    let vesting_account = Pubkey::from_str(vesting_account.as_str()).unwrap();
    let custody_token_account = Pubkey::from_str(custody_token_account.as_str()).unwrap();
    let rest_recipient = Pubkey::from_str(rest_recipient.as_str()).unwrap();
    let owner = Pubkey::from_str(owner.as_str()).unwrap();

    let ix = crate::instructions::close_vesting_account(
        program_id,
        vesting_account,
        custody_token_account,
        rest_recipient,
        owner,
    )
    .unwrap();

    JsValue::from_serde(&ix).unwrap()
}

#[wasm_bindgen]
pub fn claim_tokens(
    program_id: String,
    vesting_account: String,
    custody_token_account: String,
    recipient: String,
    owner: String,
) -> JsValue {
    let program_id = Pubkey::from_str(program_id.as_str()).unwrap();
    let vesting_account = Pubkey::from_str(vesting_account.as_str()).unwrap();
    let custody_token_account = Pubkey::from_str(custody_token_account.as_str()).unwrap();
    let recipient = Pubkey::from_str(recipient.as_str()).unwrap();
    let owner = Pubkey::from_str(owner.as_str()).unwrap();

    let ix = crate::instructions::claim_tokens(
        program_id,
        vesting_account,
        custody_token_account,
        recipient,
        owner,
    )
    .unwrap();

    JsValue::from_serde(&ix).unwrap()
}

#[wasm_bindgen]
pub fn transfer_ownership(
    program_id: String,
    vesting_account: String,
    owner: String,
    new_owner: String,
) -> JsValue {
    let program_id = Pubkey::from_str(program_id.as_str()).unwrap();
    let vesting_account = Pubkey::from_str(vesting_account.as_str()).unwrap();
    let owner = Pubkey::from_str(owner.as_str()).unwrap();
    let new_owner = Pubkey::from_str(new_owner.as_str()).unwrap();

    let ix = crate::instructions::transfer_ownership(program_id, vesting_account, owner, new_owner)
        .unwrap();

    JsValue::from_serde(&ix).unwrap()
}

#[wasm_bindgen]
pub fn custody_signer_address(program_id: String) -> Vec<u8> {
    let program_id = Pubkey::from_str(program_id.as_str()).unwrap();

    VestingCustodySigner::key(None, &program_id)
        .to_bytes()
        .to_vec()
}

#[wasm_bindgen]
pub fn parse_vesting_account(data: Vec<u8>) -> JsValue {
    JsValue::from_serde(&VestingAccountData::try_from_slice(data.as_slice()).unwrap()).unwrap()
}
