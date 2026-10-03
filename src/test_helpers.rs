#![cfg(test)]

//! Shared setup helpers for `test.rs` and `error_paths.rs`.

use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger as _, MockAuth, MockAuthInvoke,
    },
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, Val,
};

use crate::{
    storage::{DataKey, SECONDS_PER_LEDGER},
    Contract, ContractClient,
};

/// Seconds in a day, for readable test deadlines.
pub const DAY_SECONDS: u64 = 24 * 60 * 60;

/// The default invoice total used by the setup helpers.
pub const TOTAL: i128 = 1_000;

/// Registers the contract and returns its address and a client for it.
pub fn setup(env: &Env) -> (Address, ContractClient<'_>) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    (contract_id, client)
}

/// A synthetic 32-byte details hash. Tests never use a value derived from a
/// real person or a real invoice document.
pub fn synthetic_hash(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// Converts a value to a `Val` with the target type pinned, for building
/// expected event data maps (plain `into_val` is ambiguous there).
pub fn to_val<T: IntoVal<Env, Val>>(env: &Env, value: T) -> Val {
    value.into_val(env)
}

/// Registers a Stellar Asset Contract for tests and returns its address.
pub fn setup_token(env: &Env) -> Address {
    let admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(admin).address()
}

/// Mints test tokens. Requires mocked auth (the SAC admin's signature).
pub fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

/// Creates an invoice for `freelancer` with the default total and a deadline
/// one day out. `client_opt` optionally restricts who may pay. Requires
/// mocked auth for the freelancer (use `env.mock_all_auths` in the test).
pub fn setup_invoice(
    env: &Env,
    client: &ContractClient<'_>,
    freelancer: &Address,
    token: &Address,
    client_opt: Option<&Address>,
) -> u64 {
    let now = env.ledger().timestamp();
    client.create_invoice(
        freelancer,
        token,
        &TOTAL,
        &(now + DAY_SECONDS),
        &client_opt.cloned(),
        &synthetic_hash(env, 0xB2),
    )
}

/// Moves the ledger forward by `seconds`, keeping sequence and timestamp in
/// step (at roughly 5 seconds per ledger).
pub fn advance_time(env: &Env, seconds: u64) {
    let ledgers = u32::try_from(seconds / SECONDS_PER_LEDGER).unwrap();
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + ledgers);
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + seconds);
}

/// Remaining TTL of a persistent entry, read inside the contract's context.
pub fn persistent_ttl(env: &Env, contract_id: &Address, key: &DataKey) -> u32 {
    env.as_contract(contract_id, || env.storage().persistent().get_ttl(key))
}

/// Remaining TTL of the contract instance.
pub fn instance_ttl(env: &Env, contract_id: &Address) -> u32 {
    env.as_contract(contract_id, || env.storage().instance().get_ttl())
}

/// Mocks only the payer's signature for one `pay` invocation.
pub fn mock_payer_auth_for_pay(
    env: &Env,
    contract_id: &Address,
    payer: &Address,
    invoice_id: u64,
    amount: i128,
) {
    env.mock_auths(&[MockAuth {
        address: payer,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "pay",
            args: (invoice_id, payer.clone(), amount).into_val(env),
            sub_invokes: &[],
        },
    }]);
}
