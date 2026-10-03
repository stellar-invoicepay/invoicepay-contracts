//! Contract error codes, stored types, and events.
//!
//! Codes are grouped into ranges by category so that new variants can be added
//! without renumbering the ones that already exist:
//!
//! ```text
//! 1-9    Lookup
//! 10-29  Lifecycle & timing
//! 30-49  Validation
//! ```
//!
//! Every variant here has exactly one row in `ERRORS.md` at the repository
//! root and exactly one test in `src/error_paths.rs`. `scripts/check-errors.mjs`
//! fails CI if this enum and `ERRORS.md` drift apart.

use soroban_sdk::{contracterror, contractevent, contracttype, Address, BytesN, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // 1-9: Lookup
    /// No invoice record exists for the given id.
    InvoiceNotFound = 1,
    /// No payment record exists for the given invoice and payer.
    PaymentNotFound = 2,
    // 10-29: Lifecycle & timing
    /// The freelancer has cancelled the invoice.
    InvoiceCancelled = 10,
    /// The invoice's due date (`due_at`) has passed; it no longer accepts payments.
    InvoiceExpired = 11,
    /// The payment would take the invoice past its total amount.
    Overpayment = 12,
    /// The invoice is restricted to one client and the payer does not match.
    ClientMismatch = 13,
    /// The invoice cannot be cancelled because a payment has been made.
    CancelNotAllowed = 14,
    /// The refund would exceed what that payer has paid, net of refunds.
    RefundTooLarge = 15,
    /// The invoice already has the maximum number of distinct payers
    /// ([`crate::invoices::MAX_PAYERS_PER_INVOICE`]).
    PayersTooMany = 16,
    // 30-49: Validation
    /// The amount given was zero or negative.
    InvalidAmount = 30,
    /// `create_invoice` was called with a due date that is not in the future.
    DueAtInPast = 31,
}

/// An invoice recorded on-chain.
///
/// The invoice document itself never touches the chain: `details_hash` is the
/// SHA-256 of the off-chain document (no names, invoice numbers or contact
/// details). The contract never holds funds — see
/// `docs/decisions/0001-no-custody-token-flow.md`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    /// Invoice id, starting at 1.
    pub id: u64,
    /// The freelancer who issued the invoice and receives payment.
    pub freelancer: Address,
    /// The SEP-41 token (typically a USDC testline) the invoice is denominated in.
    pub token: Address,
    /// Total amount invoiced.
    pub amount: i128,
    /// Sum of all payments ever made, gross of refunds. Never decreases; it
    /// is what makes `cancel` permanent once anything has been paid.
    pub paid_total: i128,
    /// Sum of all refunds ever made by the freelancer.
    pub refunded_total: i128,
    /// Payment deadline, in Unix seconds (the same clock as
    /// `env.ledger().timestamp()`). Payments fail after it; refunds do not.
    pub due_at: u64,
    /// When set, only this address may call `pay`.
    pub client_opt: Option<Address>,
    /// Opaque hash of the off-chain invoice document.
    pub details_hash: BytesN<32>,
    /// Set by `cancel`; a cancelled invoice accepts no payments.
    pub cancelled: bool,
}

/// The derived state of an invoice, returned by `get_invoice` and `receipt`.
///
/// It is never stored: it is computed from the invoice record on every read,
/// so it cannot drift from the totals.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InvoiceStatus {
    /// Nothing paid yet (or less than the total, net of refunds).
    Open,
    /// Paid in full: net payments cover `amount`.
    Paid,
    /// Cancelled by the freelancer; no payments accepted.
    Cancelled,
}

/// What one payer has moved, gross and net.
///
/// Both fields are kept forever: the paid history is the provable record this
/// contract exists for, so a refund never rewrites it.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayerRecord {
    /// Total this payer has paid into the invoice.
    pub paid: i128,
    /// Total the freelancer has refunded to this payer.
    pub refunded: i128,
}

/// One payer's line on a [`Receipt`].
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    /// The payer.
    pub payer: Address,
    /// Gross amount this payer paid.
    pub paid: i128,
    /// Gross amount this payer has been refunded.
    pub refunded: i128,
}

/// The full payment record for one invoice, as returned by `receipt`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    /// The invoice the receipt belongs to.
    pub invoice_id: u64,
    /// Total amount invoiced.
    pub amount: i128,
    /// Sum of all payments, gross of refunds.
    pub paid_total: i128,
    /// Sum of all refunds.
    pub refunded_total: i128,
    /// Derived status ([`InvoiceStatus`]).
    pub status: InvoiceStatus,
    /// One entry per distinct payer, bounded by
    /// [`crate::invoices::MAX_PAYERS_PER_INVOICE`].
    pub payments: Vec<Payment>,
}

/// Emitted by `create_invoice` once an invoice is recorded.
///
/// The payer restriction (`client_opt`) is deliberately not in the event data:
/// it is stored with the invoice and readable via `get_invoice`; the event
/// stays a compact creation record.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceCreated {
    /// The new invoice's id.
    #[topic]
    pub invoice_id: u64,
    /// The freelancer who created the invoice.
    pub freelancer: Address,
    /// The token the invoice is denominated in.
    pub token: Address,
    /// Total amount invoiced.
    pub amount: i128,
    /// Payment deadline, in Unix seconds.
    pub due_at: u64,
    /// Opaque hash of the off-chain invoice document.
    pub details_hash: BytesN<32>,
}

/// Emitted by `pay` after tokens have moved from the payer to the freelancer.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoicePaid {
    /// The invoice that was paid.
    #[topic]
    pub invoice_id: u64,
    /// The address that made the payment.
    #[topic]
    pub payer: Address,
    /// The amount of this payment.
    pub amount: i128,
    /// The invoice's `paid_total` after this payment.
    pub total_paid: i128,
}

/// Emitted by `cancel` after the freelancer cancels the invoice.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceCancelled {
    /// The cancelled invoice's id.
    #[topic]
    pub invoice_id: u64,
    /// The freelancer who cancelled it.
    pub freelancer: Address,
}

/// Emitted by `refund` after tokens have moved back from the freelancer.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvoiceRefunded {
    /// The invoice the refund belongs to.
    #[topic]
    pub invoice_id: u64,
    /// The payer who received the refund.
    #[topic]
    pub payer: Address,
    /// The amount of this refund.
    pub amount: i128,
    /// The invoice's `refunded_total` after this refund.
    pub total_refunded: i128,
}
