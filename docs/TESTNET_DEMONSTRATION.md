# invoicepay synthetic testnet demonstration

Deployed October 8, 2026. Reverified `2026-10-08T18:52:08.061Z`.
The maintainer explicitly authorized this demonstration using synthetic data.
No real pilot, partner agreement, security audit or production readiness is claimed.

- Network: Stellar **testnet**, protocol 29 at verification.
- Contract ID: `CCJ652VR7Y7H5KE5FUDT4FSTTN3FJECY44RJYHBW4J2BPSXVETJLTJKU`.
- [Stellar Expert contract](https://stellar.expert/explorer/testnet/contract/CCJ652VR7Y7H5KE5FUDT4FSTTN3FJECY44RJYHBW4J2BPSXVETJLTJKU).
- [Stellar Lab contract explorer](https://lab.stellar.org/r/testnet/contract/CCJ652VR7Y7H5KE5FUDT4FSTTN3FJECY44RJYHBW4J2BPSXVETJLTJKU).
- Source commit: `02992e1ab48b92372248c642edbc45cf1286bc08`; runtime source matched the rebuilt artifact. The worktree also contains contributor documentation changes.
- Wasm SHA-256: `df77525063bda9e74fd183547895be59a3c7efd50b039cc8241d4313b316f709`. Local and deployed hashes matched.
- Public deployer account: `GBHVPV4S3JRAOPQYODNULNPBW57REIJ2SDNAHYHN6SXGOKJCS3XLPVZD`. Signing material stays outside Git.

| Transaction | Hash / explorer | RPC verification |
|---|---|---|
| creation | [ab1b0d5718a824d718996c6d9497ac56d93770897769ff19435af37b3618eb2e](https://stellar.expert/explorer/testnet/tx/ab1b0d5718a824d718996c6d9497ac56d93770897769ff19435af37b3618eb2e) | SUCCESS; ledger 5078370 |
| upload | [a3094be2490101787aabd570f41ee985e26f97f8db49551fdb05a0c51710f7cd](https://stellar.expert/explorer/testnet/tx/a3094be2490101787aabd570f41ee985e26f97f8db49551fdb05a0c51710f7cd) | SUCCESS; ledger 5078368 |

## Observed checks

Before deployment, formatting, locked clippy with warnings denied, locked Rust tests,
Node checker tests, error/documentation checks and the Stellar CLI 28.1.0 Wasm build passed.
30 Rust tests and 9 Node tests.
During verification, all listed transactions were `SUCCESS` through the testnet RPC
and the deployed Wasm hash matched the tested local artifact.

This contract has no initializer.

## Remaining boundaries

[Live synthetic CLI/RPC smoke tests](LIVE_TESTNET_SMOKE.md) have now passed.
Browser-wallet integration has not been tested.
Local tests do not establish real-user outcomes. No mainnet deployment is authorized.
Pilot deployment still requires the documented real-user agreement. Testnet resets
and storage expiry can make this address or its state unavailable later.
