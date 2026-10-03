#![cfg(test)]

//! Happy-path and integration tests. Error paths live in `error_paths.rs`.

use super::*;
use soroban_sdk::{
    map,
    testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke},
    token::TokenClient,
    vec, Address, Env, IntoVal, Symbol,
};

use crate::storage::{DataKey, MIN_TTL_LEDGERS, SECONDS_PER_LEDGER};
use crate::test_helpers::{
    advance_time, instance_ttl, mint, mock_payer_auth_for_pay, persistent_ttl, setup,
    setup_invoice, setup_token, synthetic_hash, to_val, DAY_SECONDS, TOTAL,
};
use crate::types::InvoiceStatus;

#[test]
fn create_invoice_records_the_invoice() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);
    let details_hash = synthetic_hash(&env, 0xB2);
    let due_at = env.ledger().timestamp() + 7 * DAY_SECONDS;

    let invoice_id =
        client.create_invoice(&freelancer, &token, &TOTAL, &due_at, &None, &details_hash);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.freelancer, freelancer);
    assert_eq!(invoice.token, token);
    assert_eq!(invoice.amount, TOTAL);
    assert_eq!(invoice.paid_total, 0);
    assert_eq!(invoice.refunded_total, 0);
    assert_eq!(invoice.due_at, due_at);
    assert_eq!(invoice.client_opt, None);
    assert_eq!(invoice.details_hash, details_hash);
    assert!(!invoice.cancelled);

    // Ids are sequential.
    let second = client.create_invoice(
        &freelancer,
        &token,
        &TOTAL,
        &(env.ledger().timestamp() + 7 * DAY_SECONDS),
        &None,
        &synthetic_hash(&env, 0xB3),
    );
    assert_eq!(second, 2);
}

#[test]
fn create_invoice_extends_the_instance_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);

    setup_invoice(&env, &client, &freelancer, &token, None);

    let ttl = instance_ttl(&env, &contract_id);
    assert!(
        ttl >= storage::INSTANCE_TTL_EXTEND_TO - 1,
        "expected the instance TTL to be extended to at least {} ledgers, got {}",
        storage::INSTANCE_TTL_EXTEND_TO,
        ttl
    );
}

#[test]
fn create_invoice_extends_the_invoice_record_ttl_toward_its_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);
    let now = env.ledger().timestamp();
    let due_at = now + 90 * DAY_SECONDS;

    let invoice_id = client.create_invoice(
        &freelancer,
        &token,
        &TOTAL,
        &due_at,
        &None,
        &synthetic_hash(&env, 0xB2),
    );

    let ttl = persistent_ttl(&env, &contract_id, &DataKey::Invoice(invoice_id));
    // The record must reach at least the 7-day floor...
    assert!(
        ttl >= MIN_TTL_LEDGERS - 1,
        "expected the invoice TTL to reach at least {} ledgers, got {}",
        MIN_TTL_LEDGERS,
        ttl
    );
    // ...and stay under the full deadline horizon, which is far larger.
    let horizon_ledgers =
        u32::try_from((due_at + storage::SETTLEMENT_MARGIN_SECONDS - now) / SECONDS_PER_LEDGER)
            .unwrap_or(u32::MAX);
    assert!(
        ttl <= horizon_ledgers,
        "expected the invoice TTL to stay under the {}-ledger horizon, got {}",
        horizon_ledgers,
        ttl
    );
}

#[test]
fn pay_moves_tokens_straight_to_the_freelancer() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    let payer_token = TokenClient::new(&env, &token);
    let freelancer_before = payer_token.balance(&freelancer);
    client.pay(&invoice_id, &payer, &400);

    // The contract is not a party to the transfer: the tokens moved directly
    // from payer to freelancer.
    assert_eq!(payer_token.balance(&payer), TOTAL - 400);
    assert_eq!(payer_token.balance(&freelancer), freelancer_before + 400);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.paid_total, 400);
    assert_eq!(invoice.refunded_total, 0);
}

#[test]
fn partial_payments_sum_to_the_total() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.pay(&invoice_id, &payer, &300);
    client.pay(&invoice_id, &payer, &300);
    client.pay(&invoice_id, &payer, &400);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.paid_total, TOTAL);

    let receipt = client.receipt(&invoice_id);
    assert_eq!(receipt.status, InvoiceStatus::Paid);
    assert_eq!(receipt.payments.len(), 1);
    assert_eq!(receipt.payments.get(0).unwrap().paid, TOTAL);
    assert_eq!(receipt.payments.get(0).unwrap().refunded, 0);
}

#[test]
fn two_payers_appear_as_two_receipt_lines() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &first, TOTAL);
    mint(&env, &token, &second, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.pay(&invoice_id, &first, &250);
    client.pay(&invoice_id, &second, &250);
    client.pay(&invoice_id, &first, &250);

    let receipt = client.receipt(&invoice_id);
    assert_eq!(receipt.payments.len(), 2);
    assert_eq!(receipt.paid_total, 750);

    let first_line = receipt.payments.iter().find(|p| p.payer == first).unwrap();
    assert_eq!(first_line.paid, 500);
    let second_line = receipt.payments.iter().find(|p| p.payer == second).unwrap();
    assert_eq!(second_line.paid, 250);
}

#[test]
fn an_invoice_with_a_named_client_only_accepts_that_client() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let named_client = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &named_client, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, Some(&named_client));

    // The named client can pay.
    client.pay(&invoice_id, &named_client, &100);
    assert_eq!(client.get_invoice(&invoice_id).paid_total, 100);
}

#[test]
fn payment_after_the_deadline_fails_but_refund_still_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.pay(&invoice_id, &payer, &500);
    // Move past the deadline (1 day) but stay inside the record's TTL.
    advance_time(&env, 31 * DAY_SECONDS);

    // Payments are closed after the deadline...
    let late_payer = Address::generate(&env);
    mint(&env, &token, &late_payer, TOTAL);
    assert_eq!(
        client.try_pay(&invoice_id, &late_payer, &100),
        Err(Ok(Error::InvoiceExpired))
    );

    // ...but returning money must stay possible.
    client.refund(&invoice_id, &payer, &200);
    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.paid_total, 500);
    assert_eq!(invoice.refunded_total, 200);
}

#[test]
fn cancel_only_works_before_any_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);

    // Nothing paid: cancel succeeds.
    let fresh = setup_invoice(&env, &client, &freelancer, &token, None);
    client.cancel(&fresh);
    let invoice = client.get_invoice(&fresh);
    assert!(invoice.cancelled);
    assert_eq!(client.receipt(&fresh).status, InvoiceStatus::Cancelled);

    // Anything paid: cancel is rejected and the invoice stays a receipt.
    let touched = setup_invoice(&env, &client, &freelancer, &token, None);
    client.pay(&touched, &payer, &10);
    client.refund(&touched, &payer, &10);
    // Even fully refunded, the payment history is permanent.
    assert_eq!(
        client.try_cancel(&touched),
        Err(Ok(Error::CancelNotAllowed))
    );
    assert!(!client.get_invoice(&touched).cancelled);
}

#[test]
fn refunds_come_from_the_freelancers_own_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    // Fund the freelancer so a refund is possible at all.
    mint(&env, &token, &freelancer, 500);

    client.pay(&invoice_id, &payer, &400);
    client.refund(&invoice_id, &payer, &150);

    let payer_token = TokenClient::new(&env, &token);
    assert_eq!(payer_token.balance(&payer), TOTAL - 400 + 150);
    assert_eq!(payer_token.balance(&freelancer), 500 + 400 - 150);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.paid_total, 400);
    assert_eq!(invoice.refunded_total, 150);

    let receipt = client.receipt(&invoice_id);
    assert_eq!(receipt.status, InvoiceStatus::Open);
    assert_eq!(receipt.payments.get(0).unwrap().paid, 400);
    assert_eq!(receipt.payments.get(0).unwrap().refunded, 150);
}

#[test]
fn pay_extends_the_record_ttls() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.pay(&invoice_id, &payer, &100);

    let ttl = persistent_ttl(&env, &contract_id, &DataKey::Payer(invoice_id, payer));
    assert!(
        ttl >= MIN_TTL_LEDGERS - 1,
        "expected the payer record TTL to reach at least {} ledgers, got {}",
        MIN_TTL_LEDGERS,
        ttl
    );
    let payers_ttl = persistent_ttl(&env, &contract_id, &DataKey::Payers(invoice_id));
    assert!(
        payers_ttl >= MIN_TTL_LEDGERS - 1,
        "expected the payer-list TTL to reach at least {} ledgers, got {}",
        MIN_TTL_LEDGERS,
        payers_ttl
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn create_invoice_requires_the_freelancer_signature() {
    let env = Env::default();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);

    client.create_invoice(
        &freelancer,
        &token,
        &TOTAL,
        &(env.ledger().timestamp() + DAY_SECONDS),
        &None,
        &synthetic_hash(&env, 0xB2),
    );
}

/// Creates the invoice with only the freelancer's signature mocked, so later
/// calls run against a real invoice with no blanket auth bypass.
fn setup_invoice_with_only_freelancer_auth(
    env: &Env,
    contract_id: &Address,
    client: &ContractClient<'_>,
    freelancer: &Address,
    token: &Address,
) -> u64 {
    let amount = 1_000_i128;
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    let details_hash = synthetic_hash(env, 0xB2);

    // Mocks only the freelancer's signature for this `create_invoice`
    // invocation, so later calls run with no blanket auth bypass.
    env.mock_auths(&[MockAuth {
        address: freelancer,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "create_invoice",
            args: (
                freelancer.clone(),
                token.clone(),
                amount,
                due_at,
                Option::<Address>::None,
                details_hash.clone(),
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);

    client.create_invoice(freelancer, token, &amount, &due_at, &None, &details_hash)
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn pay_requires_the_payer_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    let invoice_id =
        setup_invoice_with_only_freelancer_auth(&env, &contract_id, &client, &freelancer, &token);

    // Only the freelancer's signature was mocked for the creation call; the
    // payer has not authorized this payment.
    client.pay(&invoice_id, &payer, &100);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn cancel_requires_the_freelancer_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    let invoice_id =
        setup_invoice_with_only_freelancer_auth(&env, &contract_id, &client, &freelancer, &token);

    // The payer's signature is mocked for the payment...
    mock_payer_auth_for_pay(&env, &contract_id, &payer, invoice_id, 100);
    client.pay(&invoice_id, &payer, &100);

    // ...but nobody has authorized this cancel.
    client.cancel(&invoice_id);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn refund_requires_the_freelancer_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    let invoice_id =
        setup_invoice_with_only_freelancer_auth(&env, &contract_id, &client, &freelancer, &token);

    mock_payer_auth_for_pay(&env, &contract_id, &payer, invoice_id, 100);
    client.pay(&invoice_id, &payer, &100);

    // Nobody has authorized this refund.
    client.refund(&invoice_id, &payer, &50);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn refund_rejects_a_signature_from_someone_other_than_the_freelancer() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let stranger = Address::generate(&env);
    let token = setup_token(&env);
    let invoice_id =
        setup_invoice_with_only_freelancer_auth(&env, &contract_id, &client, &freelancer, &token);

    mock_payer_auth_for_pay(&env, &contract_id, &payer, invoice_id, 100);
    client.pay(&invoice_id, &payer, &100);

    // A stranger authorizes the refund invocation, but the contract demands
    // the freelancer's signature, so the call must still fail.
    env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "refund",
            args: (invoice_id, payer.clone(), 50_i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.refund(&invoice_id, &payer, &50);
}

#[test]
fn lifecycle_publishes_documented_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let details_hash = synthetic_hash(&env, 0xB2);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;

    // `env.events().all()` returns the events of the last invocation, so each
    // step is asserted right after its call, filtered to this contract.
    let invoice_id =
        client.create_invoice(&freelancer, &token, &TOTAL, &due_at, &None, &details_hash);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "invoice_created"), invoice_id).into_val(&env),
                map![
                    &env,
                    (Symbol::new(&env, "amount"), to_val(&env, TOTAL)),
                    (
                        Symbol::new(&env, "details_hash"),
                        to_val(&env, details_hash.clone())
                    ),
                    (Symbol::new(&env, "due_at"), to_val(&env, due_at)),
                    (
                        Symbol::new(&env, "freelancer"),
                        to_val(&env, freelancer.clone())
                    ),
                    (Symbol::new(&env, "token"), to_val(&env, token.clone())),
                ]
                .into_val(&env),
            ),
        ]
    );
    client.pay(&invoice_id, &payer, &400);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "invoice_paid"), invoice_id, payer.clone()).into_val(&env),
                map![
                    &env,
                    (Symbol::new(&env, "amount"), to_val(&env, 400_i128)),
                    (Symbol::new(&env, "total_paid"), to_val(&env, 400_i128)),
                ]
                .into_val(&env),
            ),
        ]
    );

    // A paid invoice cannot be cancelled, so the cancel event is asserted on
    // a fresh one.
    let fresh = client.create_invoice(
        &freelancer,
        &token,
        &100,
        &(env.ledger().timestamp() + DAY_SECONDS),
        &None,
        &synthetic_hash(&env, 0xB3),
    );
    client.cancel(&fresh);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "invoice_cancelled"), fresh).into_val(&env),
                map![
                    &env,
                    (
                        Symbol::new(&env, "freelancer"),
                        to_val(&env, freelancer.clone())
                    ),
                ]
                .into_val(&env),
            ),
        ]
    );
}

#[test]
fn refund_publishes_the_documented_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.pay(&invoice_id, &payer, &400);
    client.refund(&invoice_id, &payer, &150);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "invoice_refunded"),
                    invoice_id,
                    payer.clone()
                )
                    .into_val(&env),
                map![
                    &env,
                    (Symbol::new(&env, "amount"), to_val(&env, 150_i128)),
                    (Symbol::new(&env, "total_refunded"), to_val(&env, 150_i128)),
                ]
                .into_val(&env),
            ),
        ]
    );
}
