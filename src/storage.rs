//! Storage keys, TTL constants, and TTL-extension helpers.
//!
//! Contract-wide values (the invoice-id counter) live in instance storage.
//! Each invoice, per-payer record and per-invoice payer list gets its own
//! persistent entry, extended from the invoice's real due date (`due_at`)
//! plus a safety margin.
//!
//! Read the State Archival guide before changing anything here:
//! <https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival>

use soroban_sdk::{contracttype, Address, Env};

/// Stellar ledgers close roughly every 5 seconds, so a day is about 17,280
/// ledgers. TTL values are expressed in ledgers.
pub const DAY_IN_LEDGERS: u32 = 17_280;

/// The same 5-second assumption as [`DAY_IN_LEDGERS`], in seconds — used to
/// turn real deadlines into ledger counts.
pub const SECONDS_PER_LEDGER: u64 = 5;

/// How long an invoice's records must outlive its due date: a safety margin
/// that covers late refunds and dispute lookups after the deadline.
pub const SETTLEMENT_MARGIN_SECONDS: u64 = 30 * 24 * 60 * 60;

/// The smallest TTL bump a record gets, even after its deadline has passed.
pub const MIN_TTL_LEDGERS: u32 = 7 * DAY_IN_LEDGERS;

/// Instance storage is extended when its remaining TTL falls below this.
pub const INSTANCE_TTL_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;

/// Instance storage is extended to at least this much remaining TTL.
pub const INSTANCE_TTL_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;

/// Keys for contract storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// The next invoice id to hand out; the first id is 1.
    NextInvoiceId,
    /// One invoice record, by id.
    Invoice(u64),
    /// One payer's paid/refunded record, by (invoice, payer).
    Payer(u64, Address),
    /// One invoice's list of distinct payers, so `receipt` has a bounded,
    /// keyed place to start. Length is capped by
    /// [`crate::invoices::MAX_PAYERS_PER_INVOICE`].
    Payers(u64),
}

/// Extends the TTL of the contract instance and its instance entries.
///
/// Call this whenever instance storage is read or written, so contract-wide
/// values do not archive. A flat threshold is correct here because these
/// values have no natural deadline.
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

/// Ledgers of headroom wanted for a record that must outlive `due_at`.
///
/// Computed from the record's real deadline plus [`SETTLEMENT_MARGIN_SECONDS`],
/// floored so that late access still gets a useful bump, and capped by the
/// network maximum TTL.
pub fn record_ttl_target(env: &Env, due_at: u64) -> u32 {
    let now = env.ledger().timestamp();
    let deadline = due_at.saturating_add(SETTLEMENT_MARGIN_SECONDS);
    let remaining_secs = deadline.saturating_sub(now);
    let wanted = remaining_secs / SECONDS_PER_LEDGER;
    let wanted = u32::try_from(wanted).unwrap_or(u32::MAX);
    wanted.max(MIN_TTL_LEDGERS).min(env.storage().max_ttl())
}

/// Extends a persistent record's TTL toward `due_at + SETTLEMENT_MARGIN_SECONDS`.
///
/// Tops the entry up when less than half of that horizon remains. Call this
/// whenever an invoice, payer record or payer list entry is read or written.
pub fn extend_record_ttl(env: &Env, key: &DataKey, due_at: u64) {
    let target = record_ttl_target(env, due_at);
    env.storage()
        .persistent()
        .extend_ttl(key, target / 2, target);
}
