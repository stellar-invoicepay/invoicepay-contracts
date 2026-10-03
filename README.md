# invoicepay — contracts

Soroban contract for **invoicepay**: freelancers record invoices and get paid
in stable tokens, with the contract **never holding funds** — payments move
token straight from the payer to the freelancer. Part of a three-repo project
with `invoicepay-app` (web app) and `invoicepay-docs` (mdBook book).

**Status: v0 contract built and locally verified. Not deployed. No pilot has
happened. Nothing here has run against a live network.**

## What the contract does

- `create_invoice(freelancer, token, amount, due_at, client_opt, details_hash)`:
  the freelancer records an invoice denominated in one SEP-41 token (typically
  a USDC testline), with a payment deadline and an opaque `details_hash` — the
  SHA-256 of the off-chain invoice document. `client_opt` optionally restricts
  who may pay. Freelancer-authorized. Returns the invoice id.
- `pay(invoice_id, payer, amount)`: moves tokens from the payer straight to
  the freelancer. Partial payments allowed; overpayment, cancelled and
  expired invoices rejected; the payer must match `client_opt` when one was
  set.
- `cancel(invoice_id)`: freelancer-authorized, only while nothing has ever
  been paid — an invoice that has had any payment stays on-chain as a receipt.
- `refund(invoice_id, payer, amount)`: freelancer-authorized, paid back from
  the freelancer's own balance, capped at what that payer has paid net of
  earlier refunds. Allowed after the deadline.
- `get_invoice(invoice_id)`, `receipt(invoice_id)` (totals, derived status,
  one payment line per distinct payer, bounded by `MAX_PAYERS_PER_INVOICE`).
- **The contract never holds a balance.** It is a registry plus direct token
  transfers; the reasoning and the alternatives (including existing Stellar
  escrow and pay-URI prior art) are recorded in
  [docs/decisions/0001-no-custody-token-flow.md](docs/decisions/0001-no-custody-token-flow.md).

## Privacy

No names, invoice numbers or contact details touch the chain. The invoice
document exists off-chain only; on-chain it is represented by `details_hash`.
Test fixtures use synthetic values only.

## Structure

Standard layout, shared with this program's other contract repos: thin
[`src/lib.rs`](src/lib.rs) (`#[contractimpl]` delegation only),
[`src/types.rs`](src/types.rs) (error enum in numbered ranges, stored types,
`#[contractevent]` events), [`src/storage.rs`](src/storage.rs) (keys and TTL
helpers computed from each invoice's real `due_at` deadline),
[`src/invoices.rs`](src/invoices.rs) (logic),
[`src/error_paths.rs`](src/error_paths.rs) (exactly one test per error
variant), [`src/test.rs`](src/test.rs) (lifecycle, auth and event-layout
tests).

- [`ERRORS.md`](ERRORS.md) — one row per error variant; the user-facing
  wording there is the source of truth for the app.
- [`docs/events.md`](docs/events.md) — topic and data layout of all four
  events, asserted exactly in tests.
- [`docs/decisions/0001-no-custody-token-flow.md`](docs/decisions/0001-no-custody-token-flow.md)
  — why there is no escrow and no crate beyond `soroban-sdk`.
- [`scripts/check-errors.mjs`](scripts/check-errors.mjs) — fails when
  `ERRORS.md` and `enum Error` drift; `node --test` covers the checker.
- [`scripts/deploy-testnet.sh`](scripts/deploy-testnet.sh) — **written, never
  run.** Deploying is Tim's step.

## Checks

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test              # 29 tests: 11 error paths + 18 lifecycle/auth/TTL
node --test             # 9 tests over the ERRORS.md checker
node scripts/check-errors.mjs
stellar contract build  # wasm32v1-none; verified with Stellar CLI 28.1.0
```

Toolchain: `soroban-sdk = "28"` (28.0.0 in `Cargo.lock`), Rust stable
(1.84.0+), target `wasm32v1-none`, release profile with
`overflow-checks = true`.

## Deliberately not built

Optional platform fee in basis points, several accepted tokens per invoice,
pagination for a freelancer's invoices, property-based tests, an extend-TTL
entrypoint anyone can call, resource benchmarks. Each has a draft under
[docs/issue-drafts/](docs/issue-drafts/); the list is mirrored in
[ROADMAP.md](ROADMAP.md). The 250-payer cap on `receipt` is a v0 boundary from
our own no-unbounded-storage rule, not a playbook requirement. Testnet only —
no mainnet, ever, in this phase.

## License

MIT — see [LICENSE](LICENSE).
