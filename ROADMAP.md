# Roadmap

What is next for `invoicepay-contracts`, in order. Anything not listed as done is
**not implemented**.

## Status

- [x] Repository governance: AGENTS.md, CONTRIBUTING.md, ROADMAP.md, LICENSE,
      .gitignore, .gitattributes (2026-10-01).
- [ ] v0 contract from the project's playbook section.

## Next

- [ ] v0 contract from `STELLAR-BUILD-PLAYBOOK-v3.md` section 6
      (present in `~/Desktop/Drips/_reference/playbooks/`, confirmed
      2026-10-02). v0 scope from that section: Soroban contract where
      freelancers invoice clients in USDC and the contract never holds
      funds — payments move token straight from client to freelancer.
      Entrypoints: `create_invoice`, `pay` (partial payments allowed,
      overpayment rejected), `cancel` (freelancer, only before any
      payment), `refund` (freelancer pays back from their own balance),
      `get_invoice`, `receipt`. `details_hash` keeps invoice documents
      off-chain. Planned per the program stack: thin `lib.rs`; `types.rs`
      (error enum, stored types, events); `storage.rs`; `error_paths.rs`
      with one test per variant; `test.rs` lifecycle tests; `ERRORS.md` +
      `scripts/check-errors.mjs` and its tests; rust-toolchain pinned to
      `wasm32v1-none`; release profile with `overflow-checks = true`.
- [ ] CI (`contract.yml`): fmt, clippy -D warnings, cargo test, node --test
      scripts/, check-errors, `stellar contract build` (CLI v28.1.0). Lands
      with the first code that can pass it.

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
