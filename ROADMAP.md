# Roadmap

What is next for `invoicepay-contracts`, in order. Anything not listed as done is
**not implemented**.

## Status

- [x] Repository governance: AGENTS.md, CONTRIBUTING.md, ROADMAP.md, LICENSE,
      .gitignore, .gitattributes (2026-10-01).
- [x] v0 contract from the project's playbook section (2026-10-03): the six
      entrypoints, ranged error codes, four documented events, 29 tests
      including one error-path test per variant, all six local checks green
      (`cargo fmt --check`, clippy `-D warnings`, `cargo test`, `node
      --test`, `node scripts/check-errors.mjs`, `stellar contract build`).
      One v0-boundary cap added beyond the playbook: `MAX_PAYERS_PER_INVOICE
      = 250` bounds `receipt()` (no-unbounded-storage rule); recorded in the
      README and `docs/decisions/0001-no-custody-token-flow.md`.

## Next

- [ ] CI (`contract.yml`): fmt, clippy -D warnings, cargo test, node --test
      check-errors, `stellar contract build` (CLI v28.1.0). Written on
      2026-10-03; proves itself on GitHub on the next push.

## Deliberately unimplemented (from playbook section 6)

Listed there as the v0 boundary; each will get a draft issue when the v0
contract lands:

- Optional platform fee in basis points.
- Several accepted tokens per invoice.
- Pagination for a freelancer's invoices.
- Property-based tests.
- An extend-TTL entrypoint anyone can call.
- Resource benchmarks.

## Decisions needed from Tim

1. **Build standard — decided (2026-10-02).** v3 section 6 is the scope
   authority for what the contract does; v4 plus the schoolfees repos are
   the standard for how it is built (doc set, AGENTS.md, CI, checkers).

## Explicitly out of scope

Mainnet deployment. Anything the v0 design does not ask for.
