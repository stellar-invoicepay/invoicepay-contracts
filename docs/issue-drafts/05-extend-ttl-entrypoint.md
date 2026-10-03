# Add an extend-TTL entrypoint anyone can call
**Difficulty:** easy
**Labels:** good first issue | area:contracts

## Problem
Records are extended automatically whenever the contract touches them, but an
invoice nobody interacts with after its due date will eventually archive
(`due_at + 30-day margin` may be shorter than a dispute window months later).
There is no way for a third party — an indexer, an auditor — to keep a record
alive deliberately. Listed in the playbook section 6 as deliberately
unimplemented.

## Scope
- Add `extend_invoice_ttl(invoice_id)` callable by anyone: re-runs
  `extend_record_ttl` for the invoice record and (when they exist) the payer
  list and payer records, using the existing `due_at`-derived target.
- No authorization needed; TTL extension only costs the caller fee, never
  changes state.
- Document the entrypoint in the README entrypoint list.

Out of scope: extending arbitrary keys, changing the TTL policy constants.

## Acceptance criteria
- [ ] Anyone can call it; a test asserts a low TTL is raised afterward.
- [ ] It never creates entries for nonexistent invoices (returns
      `InvoiceNotFound`).
- [ ] Full check suite passes.

## Where to start
`src/invoices.rs` (new function), `src/lib.rs` (entrypoint),
`src/storage.rs` (`record_ttl_target`).

## How to test
```
cargo test
```
