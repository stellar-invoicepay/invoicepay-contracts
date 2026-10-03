#![no_std]

// Keep this file thin: `#[contract]` and `#[contractimpl]` only. Logic,
// types, and storage rules live in the modules below.
mod invoices;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::types::{Error, Invoice, Receipt};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Records a new invoice for `freelancer`, denominated in `token`, with
    /// an opaque `details_hash` and a payment deadline (`due_at`, Unix
    /// seconds). `client_opt` optionally restricts who may pay. Returns the
    /// new invoice id. The contract never holds funds.
    ///
    /// Errors:
    /// - [`crate::types::Error::InvalidAmount`] for `amount <= 0`.
    /// - [`crate::types::Error::DueAtInPast`] when `due_at` is not in the future.
    pub fn create_invoice(
        env: Env,
        freelancer: Address,
        token: Address,
        amount: i128,
        due_at: u64,
        client_opt: Option<Address>,
        details_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        invoices::create_invoice(
            &env,
            freelancer,
            token,
            amount,
            due_at,
            client_opt,
            details_hash,
        )
    }

    /// Pays `amount` toward the invoice, moving tokens from `payer` straight
    /// to the freelancer. Partial payments are allowed.
    ///
    /// Errors: [`crate::types::Error::InvoiceNotFound`],
    /// [`crate::types::Error::InvoiceCancelled`],
    /// [`crate::types::Error::InvalidAmount`],
    /// [`crate::types::Error::InvoiceExpired`],
    /// [`crate::types::Error::ClientMismatch`],
    /// [`crate::types::Error::Overpayment`],
    /// [`crate::types::Error::PayersTooMany`].
    pub fn pay(env: Env, invoice_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
        invoices::pay(&env, invoice_id, payer, amount)
    }

    /// Cancels the invoice. Freelancer-authorized; only possible while
    /// nothing has ever been paid.
    ///
    /// Errors: [`crate::types::Error::InvoiceNotFound`],
    /// [`crate::types::Error::InvoiceCancelled`],
    /// [`crate::types::Error::CancelNotAllowed`].
    pub fn cancel(env: Env, invoice_id: u64) -> Result<(), Error> {
        invoices::cancel(&env, invoice_id)
    }

    /// Refunds a payer from the freelancer's own balance, capped at what that
    /// payer has paid net of earlier refunds. Allowed after the due date.
    ///
    /// Errors: [`crate::types::Error::InvoiceNotFound`],
    /// [`crate::types::Error::InvalidAmount`],
    /// [`crate::types::Error::PaymentNotFound`],
    /// [`crate::types::Error::RefundTooLarge`].
    pub fn refund(env: Env, invoice_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
        invoices::refund(&env, invoice_id, payer, amount)
    }

    /// Returns the invoice record.
    ///
    /// Errors: [`crate::types::Error::InvoiceNotFound`].
    pub fn get_invoice(env: Env, invoice_id: u64) -> Result<Invoice, Error> {
        invoices::get_invoice(&env, invoice_id)
    }

    /// Returns the invoice's full receipt: totals, status, and one payment
    /// line per distinct payer (bounded).
    ///
    /// Errors: [`crate::types::Error::InvoiceNotFound`].
    pub fn receipt(env: Env, invoice_id: u64) -> Result<Receipt, Error> {
        invoices::receipt(&env, invoice_id)
    }
}
