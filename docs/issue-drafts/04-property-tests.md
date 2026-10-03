# Add property-based tests for the payment invariants
**Difficulty:** hard
**Labels:** help wanted | area:contracts

## Problem
The current tests cover hand-picked cases. The core invariants —
`paid_total - refunded_total` never exceeds `amount`; `refunded_total <=
paid_total`; the sum of per-payer net records equals
`paid_total - refunded_total`; `status` never disagrees with the totals — hold
across every random sequence of valid operations, not just the scripted ones.
Listed in the playbook section 6 as deliberately unimplemented.

## Scope
- Research first: which property-testing crate works inside `#![no_std]`
  Soroban tests on this toolchain (the program standard forbids adding
  dependencies without a recorded reason). Write the choice in
  `docs/decisions/`.
- Drive random valid sequences of `pay` / `refund` (and occasional `cancel`)
  against one invoice, asserting the invariants after every step.
- Keep the generator's amounts within the tested token's supply so transfers
  never fail for out-of-band reasons.

Out of scope: fuzzing the host, replacing the deterministic suite.

## Acceptance criteria
- [ ] A decision file records the crate choice and why it fits `no_std`.
- [ ] Randomized sequences run in CI and assert the four invariants after
      every step.
- [ ] The deterministic suite is unchanged and still passes.

## Where to start
`src/test.rs`, `src/test_helpers.rs`. Read `Cargo.toml` comments on the
dependency policy first.

## How to test
```
cargo test
```
