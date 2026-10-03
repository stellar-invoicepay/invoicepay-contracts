//! Invoice lifecycle logic: creation, payment, cancellation, refunds.
//!
//! `src/lib.rs` exposes these functions through `#[contractimpl]`; everything
//! that validates input, reads or writes storage, or moves tokens lives here.
//!
//! No custody: the contract never holds a balance. `pay` moves tokens from
//! the payer straight to the freelancer and `refund` moves them from the
//! freelancer's own balance back to a payer — see
//! `docs/decisions/0001-no-custody-token-flow.md`.
//!
//! Arithmetic: amounts are validated first (`amount > 0`, `amount <=
//! remaining`, refunds capped at what a payer has paid net of refunds), and
//! the release profile builds with `overflow-checks = true` (see
//! `Cargo.toml`), so an overflow traps instead of wrapping silently.

use soroban_sdk::{token::TokenClient, Address, BytesN, Env, Vec};

use crate::storage::{extend_instance_ttl, extend_record_ttl, DataKey};
use crate::types::{
    Error, Invoice, InvoiceCancelled, InvoiceCreated, InvoicePaid, InvoiceRefunded, InvoiceStatus,
    PayerRecord, Payment, Receipt,
};

/// The most distinct payers a single invoice can record.
///
/// The playbook does not set this cap; AGENTS.md forbids unbounded storage
/// lists, and the payer list is the one entry that grows with distinct
/// addresses. 250 is far more than a freelancer-to-client invoice needs in
/// v0; raising it is a recorded decision, not a constant tweak.
pub const MAX_PAYERS_PER_INVOICE: u32 = 250;

/// Adds two amounts that have already been validated as positive and bounded.
///
/// Overflow is unreachable in practice: each successful update is bounded by
/// the invoice's `amount` (at most `i128::MAX`) or by what a payer already
/// paid, and exhausting `i128` would take more transactions than a ledger
/// sequence can ever hold. Written as a checked add so the failure mode is a
/// trap, never a silent wrap.
fn add(a: i128, b: i128) -> i128 {
    a.checked_add(b)
        .unwrap_or_else(|| panic!("amount overflow"))
}

/// Loads an invoice or reports its absence.
fn load_invoice(env: &Env, invoice_id: u64) -> Result<Invoice, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Invoice(invoice_id))
        .ok_or(Error::InvoiceNotFound)
}

/// Everything an invoice is still owed after refunds: `paid_total -
/// refunded_total`.
fn net_paid(invoice: &Invoice) -> i128 {
    invoice.paid_total - invoice.refunded_total
}

/// Derives the invoice's status from its record on every read, so the status
/// can never drift from the totals it is computed from.
fn status_of(invoice: &Invoice) -> InvoiceStatus {
    if invoice.cancelled {
        InvoiceStatus::Cancelled
    } else if net_paid(invoice) >= invoice.amount {
        InvoiceStatus::Paid
    } else {
        InvoiceStatus::Open
    }
}

/// Records a new invoice for `freelancer`, denominated in `token`, with an
/// opaque `details_hash` and a payment deadline.
///
/// `client_opt` optionally restricts who may pay. Refunds are possible after
/// the deadline; payments are not.
pub fn create_invoice(
    env: &Env,
    freelancer: Address,
    token: Address,
    amount: i128,
    due_at: u64,
    client_opt: Option<Address>,
    details_hash: BytesN<32>,
) -> Result<u64, Error> {
    freelancer.require_auth();

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    let now = env.ledger().timestamp();
    if due_at <= now {
        return Err(Error::DueAtInPast);
    }

    let counter_key = DataKey::NextInvoiceId;
    let invoice_id: u64 = env.storage().instance().get(&counter_key).unwrap_or(1);
    let next_id = invoice_id
        .checked_add(1)
        .unwrap_or_else(|| panic!("invoice id overflow"));

    let invoice = Invoice {
        id: invoice_id,
        freelancer: freelancer.clone(),
        token: token.clone(),
        amount,
        paid_total: 0,
        refunded_total: 0,
        due_at,
        client_opt: client_opt.clone(),
        details_hash: details_hash.clone(),
        cancelled: false,
    };
    let invoice_key = DataKey::Invoice(invoice_id);
    env.storage().instance().set(&counter_key, &next_id);
    env.storage().persistent().set(&invoice_key, &invoice);

    extend_instance_ttl(env);
    extend_record_ttl(env, &invoice_key, due_at);

    InvoiceCreated {
        invoice_id,
        freelancer,
        token,
        amount,
        due_at,
        details_hash,
    }
    .publish(env);

    Ok(invoice_id)
}

/// Pays `amount` toward an invoice, moving tokens from `payer` straight to
/// the freelancer. Partial payments are allowed; overpayment, cancelled and
/// expired invoices are rejected. The payer must match `client_opt` when one
/// was set at creation.
pub fn pay(env: &Env, invoice_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
    payer.require_auth();

    let mut invoice = load_invoice(env, invoice_id)?;
    if invoice.cancelled {
        return Err(Error::InvoiceCancelled);
    }
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    if env.ledger().timestamp() > invoice.due_at {
        return Err(Error::InvoiceExpired);
    }
    if let Some(client) = &invoice.client_opt {
        if *client != payer {
            return Err(Error::ClientMismatch);
        }
    }
    let remaining = invoice.amount - net_paid(&invoice);
    if amount > remaining {
        return Err(Error::Overpayment);
    }

    let payer_key = DataKey::Payer(invoice_id, payer.clone());
    let is_new_payer = !env.storage().persistent().has(&payer_key);
    let mut record: PayerRecord =
        env.storage()
            .persistent()
            .get(&payer_key)
            .unwrap_or(PayerRecord {
                paid: 0,
                refunded: 0,
            });

    if is_new_payer {
        let payers_key = DataKey::Payers(invoice_id);
        let mut payers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&payers_key)
            .unwrap_or(Vec::new(env));
        if payers.len() >= MAX_PAYERS_PER_INVOICE {
            return Err(Error::PayersTooMany);
        }
        payers.push_back(payer.clone());
        env.storage().persistent().set(&payers_key, &payers);
    }

    record.paid = add(record.paid, amount);
    env.storage().persistent().set(&payer_key, &record);

    invoice.paid_total = add(invoice.paid_total, amount);
    let invoice_key = DataKey::Invoice(invoice_id);
    env.storage().persistent().set(&invoice_key, &invoice);

    TokenClient::new(env, &invoice.token).transfer(&payer, &invoice.freelancer, &amount);

    extend_record_ttl(env, &invoice_key, invoice.due_at);
    extend_record_ttl(env, &payer_key, invoice.due_at);
    if is_new_payer {
        extend_record_ttl(env, &DataKey::Payers(invoice_id), invoice.due_at);
    }

    InvoicePaid {
        invoice_id,
        payer,
        amount,
        total_paid: invoice.paid_total,
    }
    .publish(env);

    Ok(())
}

/// Cancels an invoice. Freelancer-authorized; only possible while nothing has
/// ever been paid — an invoice that has had any payment, even one later
/// refunded in full, stays on-chain as a receipt and cannot be cancelled.
pub fn cancel(env: &Env, invoice_id: u64) -> Result<(), Error> {
    let mut invoice = load_invoice(env, invoice_id)?;
    invoice.freelancer.require_auth();

    if invoice.cancelled {
        return Err(Error::InvoiceCancelled);
    }
    if invoice.paid_total > 0 {
        return Err(Error::CancelNotAllowed);
    }

    invoice.cancelled = true;
    let invoice_key = DataKey::Invoice(invoice_id);
    env.storage().persistent().set(&invoice_key, &invoice);
    extend_record_ttl(env, &invoice_key, invoice.due_at);

    InvoiceCancelled {
        invoice_id,
        freelancer: invoice.freelancer,
    }
    .publish(env);

    Ok(())
}

/// Refunds a payer from the freelancer's own balance, capped at what that
/// payer has paid net of earlier refunds. Allowed at any time, including
/// after the due date: returning money late must stay possible.
pub fn refund(env: &Env, invoice_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
    let mut invoice = load_invoice(env, invoice_id)?;
    invoice.freelancer.require_auth();

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let payer_key = DataKey::Payer(invoice_id, payer.clone());
    let mut record: PayerRecord = env
        .storage()
        .persistent()
        .get(&payer_key)
        .ok_or(Error::PaymentNotFound)?;
    let refundable = record.paid - record.refunded;
    if amount > refundable {
        return Err(Error::RefundTooLarge);
    }

    record.refunded = add(record.refunded, amount);
    env.storage().persistent().set(&payer_key, &record);

    invoice.refunded_total = add(invoice.refunded_total, amount);
    let invoice_key = DataKey::Invoice(invoice_id);
    env.storage().persistent().set(&invoice_key, &invoice);

    TokenClient::new(env, &invoice.token).transfer(&invoice.freelancer, &payer, &amount);

    extend_record_ttl(env, &invoice_key, invoice.due_at);
    extend_record_ttl(env, &payer_key, invoice.due_at);

    InvoiceRefunded {
        invoice_id,
        payer,
        amount,
        total_refunded: invoice.refunded_total,
    }
    .publish(env);

    Ok(())
}

/// Returns the invoice record, extending its TTL.
pub fn get_invoice(env: &Env, invoice_id: u64) -> Result<Invoice, Error> {
    let invoice = load_invoice(env, invoice_id)?;
    extend_record_ttl(env, &DataKey::Invoice(invoice_id), invoice.due_at);
    Ok(invoice)
}

/// Returns the invoice's full receipt: totals, derived status, and one
/// payment line per distinct payer (bounded by [`MAX_PAYERS_PER_INVOICE`]),
/// extending every read record's TTL.
pub fn receipt(env: &Env, invoice_id: u64) -> Result<Receipt, Error> {
    let invoice = load_invoice(env, invoice_id)?;
    extend_record_ttl(env, &DataKey::Invoice(invoice_id), invoice.due_at);

    let mut payments = Vec::new(env);
    let payers_key = DataKey::Payers(invoice_id);
    if env.storage().persistent().has(&payers_key) {
        extend_record_ttl(env, &payers_key, invoice.due_at);
        let payers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&payers_key)
            .unwrap_or(Vec::new(env));
        for payer in payers.iter() {
            let payer_key = DataKey::Payer(invoice_id, payer.clone());
            extend_record_ttl(env, &payer_key, invoice.due_at);
            let record: PayerRecord = env
                .storage()
                .persistent()
                .get(&payer_key)
                .ok_or(Error::PaymentNotFound)?;
            payments.push_back(Payment {
                payer,
                paid: record.paid,
                refunded: record.refunded,
            });
        }
    }

    Ok(Receipt {
        invoice_id,
        amount: invoice.amount,
        paid_total: invoice.paid_total,
        refunded_total: invoice.refunded_total,
        status: status_of(&invoice),
        payments,
    })
}
