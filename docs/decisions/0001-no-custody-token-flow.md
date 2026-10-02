# 0001. No-custody token flow: direct transfer, no escrow, no extra crates

Date: 2026-10-03
Status: accepted

## Context

Playbook v3 section 6 scopes v0: the contract **never holds funds** — payments
move token from the client straight to the freelancer. It directs us to state
prior art plainly and not call the idea novel. This repo resolves
`soroban-sdk 28.0.0` and builds with Stellar CLI 28.1.0.

## What was checked

- **Prior art (2026-10-03, live web sources):** Stellar payments already carry
  memos and SEP-7 pay URIs, and Trustless Work publishes open-source,
  non-custodial milestone escrow infrastructure on Soroban
  (trustlesswork.com; `Trustless-Work` on GitHub). What this project adds is
  deliberately small and is **not** claimed as novel: deterministic invoice
  ids, partial payments with overpayment rejection, due dates enforced at
  pay time, on-chain status, and per-payer receipts.
- **Token movement API, from the sibling `schoolfees-contracts` reference
  (same program stack):** `soroban_sdk::token::TokenClient` — the generated
  client for the SEP-41 token interface, which covers the Stellar Asset
  Contract. `transfer(from, to, amount)` requires the `from` side's
  authorization; the test token is a real SAC via
  `env.register_stellar_asset_contract_v2` (see `src/test_helpers.rs`).
- **Dependencies:** the published OpenZeppelin Stellar crates end at 0.7.2
  and require `soroban-sdk ^26.1.0` (re-checked 2026-10-02 from the crates.io
  sparse index; see `eventbadges-contracts` decision 0001). Escrow frameworks
  are a different design (they hold funds); adopting one would contradict the
  section 6 scope.

## Decision

- **The contract is a pure invoice registry plus direct token transfers.** It
  never holds a balance: `pay` calls `token.transfer(payer -> freelancer)`
  and `refund` calls `token.transfer(freelancer -> payer)` from the
  freelancer's own balance, each guarded by that party's `require_auth`.
- **No escrow, no platform fee, no crate dependency beyond `soroban-sdk`.**
  On-chain state is limited to invoice records, per-payer paid/refunded
  records, and a per-invoice payer list for `receipt()`.
- **Off-chain data stays off-chain:** the invoice document is represented
  only by `details_hash` (SHA-256 of the document, computed off-chain). No
  names, invoice numbers or contact details touch the chain.
- Events (`invoice_created`, `invoice_paid`, `invoice_cancelled`,
  `invoice_refunded`) are `#[contractevent]` types documented in
  `docs/events.md`.

## Consequences

- Easy: one SDK major in the build; a dependency surface of exactly one
  crate; no custody responsibilities, so no treasury invariants to defend and
  the project stays out of the "holds funds" review class that
  `duestreasury` and `ajo` carry.
- Hard / accepted: **the contract cannot enforce delivery** — it records what
  was billed and paid, nothing more; if the freelancer's balance is
  insufficient, `refund` fails and the tokens stay where they are. The
  per-invoice payer list needs a size cap (`MAX_PAYERS_PER_INVOICE`) because
  AGENTS.md forbids unbounded storage growth; the cap is a v0 boundary
  documented in the README and ROADMAP, not a playbook requirement.
- What would change our mind: a pilot requirement for milestone escrow or a
  platform fee would be a new decision file, not an edit to this one.
