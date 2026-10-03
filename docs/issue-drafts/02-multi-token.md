# Accept several tokens per invoice
**Difficulty:** medium
**Labels:** help wanted | area:contracts

## Problem
An invoice is pinned to exactly one token at creation. A freelancer who would
accept USDC or another stable asset at a fixed rate must issue separate
invoices and track them apart. Listed in the playbook section 6 as
deliberately unimplemented.

## Scope
- Research first: how multi-token prices would be pinned (a fixed
  `amount_per_token` table supplied at creation vs a price oracle). Write the
  outcome in `docs/decisions/NNNN-multi-token.md` before coding; never invent
  an oracle API.
- Extend the invoice record to a small bounded set of (token, amount) pairs;
  document the cap.
- `pay` must validate the offered token against the set and keep the
  remaining-amount arithmetic correct across tokens.

Out of scope: swapping, price discovery, anything requiring an external
contract whose behavior this repo cannot verify.

## Acceptance criteria
- [ ] A decision file records the pricing approach and its risks.
- [ ] An invoice can be paid in any of its listed tokens; overpayment and
      remaining-amount rules hold per the decision file.
- [ ] Tests cover payment in a second token and rejection of an unlisted one.
- [ ] Full check suite passes.

## Where to start
`src/invoices.rs`, `src/types.rs`, `docs/decisions/0001-no-custody-token-flow.md`
(for how the single-token flow was decided).

## How to test
```
cargo test
```
