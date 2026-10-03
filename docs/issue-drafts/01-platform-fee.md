# Add an optional platform fee in basis points
**Difficulty:** medium
**Labels:** help wanted | area:contracts

## Problem
v0 moves 100% of every payment to the freelancer. A platform operator (for
example, a freelancing collective running the app) has no way to take a small,
transparent cut. Any fee taken must be visible on-chain and bounded, not a
silent skim. Listed in the playbook section 6 as deliberately unimplemented.

## Scope
- Add an optional `fee_bps` parameter to `create_invoice` (0 = no fee), with a
  hard cap constant (suggest 1,000 bps = 10%) documented in the README.
- On `pay`, split the transfer: fee share to a platform address recorded at
  creation, remainder to the freelancer. Emit the split in the `InvoicePaid`
  data so receipts stay truthful.
- Update `docs/events.md`, `ERRORS.md` (any new variants) and the app's error
  map in the same change.

Out of scope: dynamic fees, fee changes after creation, fee on refunds.

## Acceptance criteria
- [ ] `create_invoice` accepts and stores `fee_bps` with a cap; above-cap
      creation fails with a new documented error variant.
- [ ] A paid invoice's receipt shows gross, fee and net; the on-chain
      balances in a test match the split exactly.
- [ ] New error variants appear in `ERRORS.md` and `src/error_paths.rs` in the
      same commit.
- [ ] `cargo test`, `node --test` and `node scripts/check-errors.mjs` pass.

## Where to start
`src/invoices.rs` (`create_invoice`, `pay`), `src/types.rs` (invoice record,
events), `ERRORS.md`. Read the Token Interface guide first.

## How to test
```
cargo test
node scripts/check-errors.mjs
```
