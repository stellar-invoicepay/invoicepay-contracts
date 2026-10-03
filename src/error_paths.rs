#![cfg(test)]

//! Exactly one test per `Error` variant, named `error_path_<variant>` in
//! snake_case. Each test triggers the real failure path rather than
//! constructing the error value directly.
//!
//! Add a test here in the same commit that adds a variant to `src/types.rs`.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::test_helpers::{
    advance_time, mint, setup, setup_invoice, setup_token, synthetic_hash, DAY_SECONDS, TOTAL,
};

#[test]
fn error_path_invoice_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    assert_eq!(client.try_get_invoice(&99), Err(Ok(Error::InvoiceNotFound)));
    assert_eq!(client.try_receipt(&99), Err(Ok(Error::InvoiceNotFound)));
    assert_eq!(client.try_cancel(&99), Err(Ok(Error::InvoiceNotFound)));
    assert_eq!(
        client.try_pay(&99, &Address::generate(&env), &10),
        Err(Ok(Error::InvoiceNotFound))
    );
}

#[test]
fn error_path_payment_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    // Nobody has paid anything, so there is no payment record to refund.
    assert_eq!(
        client.try_refund(&invoice_id, &Address::generate(&env), &1),
        Err(Ok(Error::PaymentNotFound))
    );
}

#[test]
fn error_path_invoice_cancelled() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.cancel(&invoice_id);

    // A cancelled invoice takes no further payments, and cancelling it again
    // is an error too.
    assert_eq!(
        client.try_pay(&invoice_id, &Address::generate(&env), &10),
        Err(Ok(Error::InvoiceCancelled))
    );
    assert_eq!(
        client.try_cancel(&invoice_id),
        Err(Ok(Error::InvoiceCancelled))
    );
}

#[test]
fn error_path_invoice_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    // Move past the deadline (1 day) but stay inside the record's TTL.
    advance_time(&env, DAY_SECONDS + 60);

    let late_payer = Address::generate(&env);
    mint(&env, &token, &late_payer, TOTAL);

    assert_eq!(
        client.try_pay(&invoice_id, &late_payer, &100),
        Err(Ok(Error::InvoiceExpired))
    );
}

#[test]
fn error_path_overpayment() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.pay(&invoice_id, &payer, &TOTAL);

    // The invoice is fully paid; anything further is overpayment.
    assert_eq!(
        client.try_pay(&invoice_id, &payer, &1),
        Err(Ok(Error::Overpayment))
    );

    // And so is a single payment larger than the total.
    let second = setup_invoice(&env, &client, &freelancer, &token, None);
    assert_eq!(
        client.try_pay(&second, &payer, &(TOTAL + 1)),
        Err(Ok(Error::Overpayment))
    );
}

#[test]
fn error_path_client_mismatch() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let named_client = Address::generate(&env);
    let stranger = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &stranger, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, Some(&named_client));

    assert_eq!(
        client.try_pay(&invoice_id, &stranger, &100),
        Err(Ok(Error::ClientMismatch))
    );
}

#[test]
fn error_path_cancel_not_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    // One payment — even a small one — makes the invoice a receipt.
    client.pay(&invoice_id, &payer, &10);

    assert_eq!(
        client.try_cancel(&invoice_id),
        Err(Ok(Error::CancelNotAllowed))
    );
}

#[test]
fn error_path_refund_too_large() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    client.pay(&invoice_id, &payer, &100);

    // More than the payer's net paid total.
    assert_eq!(
        client.try_refund(&invoice_id, &payer, &101),
        Err(Ok(Error::RefundTooLarge))
    );

    // A prior refund reduces what can still be refunded.
    client.refund(&invoice_id, &payer, &60);
    assert_eq!(
        client.try_refund(&invoice_id, &payer, &41),
        Err(Ok(Error::RefundTooLarge))
    );
}

#[test]
fn error_path_payers_too_many() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);

    // Fill the payer list to its cap with tiny payments from distinct
    // addresses.
    for _ in 0..crate::invoices::MAX_PAYERS_PER_INVOICE {
        let payer = Address::generate(&env);
        mint(&env, &token, &payer, 1);
        client.pay(&invoice_id, &payer, &1);
    }

    // One more distinct payer than the cap.
    let extra = Address::generate(&env);
    mint(&env, &token, &extra, 1);
    assert_eq!(
        client.try_pay(&invoice_id, &extra, &1),
        Err(Ok(Error::PayersTooMany))
    );
}

#[test]
fn error_path_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let invoice_id = setup_invoice(&env, &client, &freelancer, &token, None);
    let now = env.ledger().timestamp();

    // create_invoice with a non-positive amount.
    assert_eq!(
        client.try_create_invoice(
            &freelancer,
            &token,
            &0,
            &(now + DAY_SECONDS),
            &None,
            &synthetic_hash(&env, 0xB2),
        ),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        client.try_create_invoice(
            &freelancer,
            &token,
            &-5,
            &(now + DAY_SECONDS),
            &None,
            &synthetic_hash(&env, 0xB2),
        ),
        Err(Ok(Error::InvalidAmount))
    );

    // pay with a non-positive amount.
    assert_eq!(
        client.try_pay(&invoice_id, &payer, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        client.try_pay(&invoice_id, &payer, &-1),
        Err(Ok(Error::InvalidAmount))
    );

    // refund with a non-positive amount.
    assert_eq!(
        client.try_refund(&invoice_id, &payer, &0),
        Err(Ok(Error::InvalidAmount))
    );
}

#[test]
fn error_path_due_at_in_past() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let freelancer = Address::generate(&env);
    let token = setup_token(&env);
    // Move off the zero timestamp so a "one second before now" case can be
    // built without underflowing.
    advance_time(&env, DAY_SECONDS);
    let now = env.ledger().timestamp();

    assert_eq!(
        client.try_create_invoice(
            &freelancer,
            &token,
            &TOTAL,
            &now,
            &None,
            &synthetic_hash(&env, 0xB2),
        ),
        Err(Ok(Error::DueAtInPast))
    );
    assert_eq!(
        client.try_create_invoice(
            &freelancer,
            &token,
            &TOTAL,
            &(now - 1),
            &None,
            &synthetic_hash(&env, 0xB2),
        ),
        Err(Ok(Error::DueAtInPast))
    );
}
