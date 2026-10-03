# Add pagination for a freelancer's invoices
**Difficulty:** medium
**Labels:** good first issue | area:contracts

## Problem
There is no way to enumerate a freelancer's invoices on-chain. The app can
only show invoices whose ids it already knows, and the id space is shared by
every freelancer. Listed in the playbook section 6 as deliberately
unimplemented.

## Scope
- Store a per-freelancer invoice-id list (persistent entry keyed by
  freelancer), appended in `create_invoice`.
- Add `invoices_of(freelancer, cursor: u32, limit: u32)` returning a bounded
  window; document the `limit` cap (suggest 50) and keep the loop bounded by
  it.
- TTL: extend the list entry from the newest invoice's `due_at`.

Out of scope: client-side indexers, deleting entries.

## Acceptance criteria
- [ ] `invoices_of` returns ids in creation order, honoring cursor and limit.
- [ ] A test creates more invoices than one page holds and walks the pages.
- [ ] The cap is documented in the README and enforced by a test.
- [ ] Full check suite passes.

## Where to start
`src/storage.rs` (new key), `src/invoices.rs`, `src/test.rs`. The eventbadges
repo's `badges_of` is the program's bounded-list reference.

## How to test
```
cargo test
```
