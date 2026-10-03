# Add resource benchmarks for the hot paths
**Difficulty:** hard
**Labels:** help wanted | area:contracts

## Problem
Nothing measures what `pay`, `refund` and `receipt` actually cost. Without
numbers, "the receipt fits in one transaction" and "the payer list cap keeps
us under budget" are beliefs, not facts — and the cap
(`MAX_PAYERS_PER_INVOICE = 250`) was chosen on judgment alone. Listed in the
playbook section 6 as deliberately unimplemented.

## Scope
- Research first: the current way to measure Soroban resource usage from
  tests (SDK test utilities or CLI tooling; verify against
  developers.stellar.org, never invent an API). Record the method in
  `docs/decisions/`.
- Benchmark `create_invoice`, `pay` (new and repeat payer), `refund` and
  `receipt` at 1, 10 and 250 payers.
- Publish the numbers and the measured ceiling for `MAX_PAYERS_PER_INVOICE`
  in the README; if 250 exceeds a budget, record the finding and lower the
  cap in a follow-up decision.

Out of scope: optimizing before measuring.

## Acceptance criteria
- [ ] A decision file records the measurement method with its source.
- [ ] Numbers exist for the four paths at three payer counts, checked into
      the repo.
- [ ] The README states the measured basis for the payer cap.

## Where to start
`src/invoices.rs` (`receipt` is the largest reader),
`src/invoices.rs::MAX_PAYERS_PER_INVOICE`, `src/test.rs` for harness patterns.

## How to test
```
cargo test
```
