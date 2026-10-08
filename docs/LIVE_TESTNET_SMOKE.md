# InvoicePay live testnet demonstration

Executed October 8, 2026, using synthetic values and two newly generated,
Friendbot-funded CLI identities. This is a real CLI/RPC business-flow smoke test
against the existing deployment, **not a browser-wallet test, real business pilot,
security audit or mainnet test**. No contract was redeployed.

Contract: [CCJ652VR7Y7H5KE5FUDT4FSTTN3FJECY44RJYHBW4J2BPSXVETJLTJKU](https://stellar.expert/explorer/testnet/contract/CCJ652VR7Y7H5KE5FUDT4FSTTN3FJECY44RJYHBW4J2BPSXVETJLTJKU).
Token: testnet native XLM SAC
`CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`, derived using the
Stellar CLI. It already existed; no token wrapping was required. Amounts below
are raw units; native XLM has seven decimals.

## Confirmed writes

Each transaction returned `SUCCESS` through testnet RPC. The actual invoice IDs
were **1** for payment/refund and **2** for unpaid cancellation.

| Action | Actual outcome | Transaction |
| --- | --- | --- |
| Create invoice 1 | Amount 10,000,000; restricted to the synthetic payer; future deadline | [a474dd5455d3694006df6da83341452986aaa9fedc3c629d1dd72d2d23b54232](https://stellar.expert/explorer/testnet/tx/a474dd5455d3694006df6da83341452986aaa9fedc3c629d1dd72d2d23b54232) |
| Partial payment | Paid 4,000,000; receipt Open | [c50fa5f1e2e0bb683c8400cc1bf01bf21e3817be0e4554aa4c2f64ae05d507e6](https://stellar.expert/explorer/testnet/tx/c50fa5f1e2e0bb683c8400cc1bf01bf21e3817be0e4554aa4c2f64ae05d507e6) |
| Refund | Refunded 1,000,000 from the freelancer; remaining 7,000,000 | [b2535e43f43f76fc395290a1e14568eb09167c3c2a4d804d51dd907291c1e288](https://stellar.expert/explorer/testnet/tx/b2535e43f43f76fc395290a1e14568eb09167c3c2a4d804d51dd907291c1e288) |
| Final payment | Additional 7,000,000; gross paid 11,000,000, refunded 1,000,000, net 10,000,000; receipt Paid | [8f4a42ade04685af1ffdf6dbf22c54cf885145b227a856e8c8226d9973bb3d36](https://stellar.expert/explorer/testnet/tx/8f4a42ade04685af1ffdf6dbf22c54cf885145b227a856e8c8226d9973bb3d36) |
| Create invoice 2 | Separate unpaid invoice, unrestricted payer | [2163f83a7d1541736b1189f851f439f55dcd207c3060cf6cc6245e69145d26d7](https://stellar.expert/explorer/testnet/tx/2163f83a7d1541736b1189f851f439f55dcd207c3060cf6cc6245e69145d26d7) |
| Cancel invoice 2 | Cancelled true; receipt Cancelled; no payments | [92e973e2e778b666623d1f72a8578935ef808755e570fc6bc73a807a637b6e15](https://stellar.expert/explorer/testnet/tx/92e973e2e778b666623d1f72a8578935ef808755e570fc6bc73a807a637b6e15) |

Read-only invoice and receipt simulations checked actual fields and exact totals
after each state change. They did not submit transactions.

## Rejected simulations

These checks used no submission and paid no transaction fees:

- Overpayment: actual `Error(Contract, #12)`.
- Wrong restricted client: actual `Error(Contract, #13)`.
- Cancellation after payment history: actual `Error(Contract, #14)`.
- Payment of a cancelled invoice: actual `Error(Contract, #10)`.
- Direct RPC `simulateTransaction` with `authMode=enforce`, payer source and no
  signatures attempting freelancer cancellation: actual
  `Error(Auth, InvalidAction)`, with diagnostic `require_auth` and the expected
  unauthorized freelancer address. This is a genuine RPC authorization check.

An earlier CLI `--auth-mode=enforce` invocation was rejected by CLI preflight
because it had no preexisting authorization entries. That preflight result is
recorded separately and is **not** counted as evidence of contract authorization.
The direct RPC check above replaces it.

## Balances and privacy

Dedicated public freelancer account:
`GBZSF3WGUUPBKGYGCFETWLSBXRCLUGFKNYXKZUFXRHL5DCILC5LWDMVB`.
Dedicated public payer account:
`GA5M5DJIF3YKJM6WDFFNJ22NCACC2YDUZZPA2NDA7ABAT5QO7G34W7LS`.
Signing material remains outside repositories and evidence.

Both accounts started with 100,000,000,000 raw units. The observed recipient
balance increases were exactly 4,000,000 for the partial payment, 1,000,000 for
the refund and 7,000,000 for the final payment. Sender debits also included
positive network/resource costs, so sender totals must not be interpreted as
transfer amounts alone. Final balances were 99,816,840,959 for the freelancer
and 99,989,004,623 for the payer. The InvoicePay contract's native token balance
stayed **0**, consistent with direct transfer and no custody.

Only synthetic opaque hashes were entered. No client names, contact details,
invoice numbers or real business data were submitted.

## Execution notes and remaining scope

Intermittent local DNS failures were worked around using a temporary loopback
relay forwarding only to the official testnet HTTPS endpoint, preserving normal
hostname/TLS verification. RPC `getNetwork` confirmed the exact testnet
passphrase and protocol 29. No host file or global network setting was changed.

The first create command failed locally because `Option<Address>` requires a
JSON-quoted address. The CLI rejected the argument before submission; its
preflight failure is preserved in the JSON evidence. The corrected command was
then executed. No ambiguous write was blindly replayed.

The browser-to-wallet flow, expiry after a real deadline, multiple distinct
payers, storage archival, provider outages, device accessibility and a real
partner pilot remain outside this smoke test. Testnet resets can remove the
records and explorer history. See [full public evidence](invoicepay-live-smoke.json)
for exact fields, transaction ledgers, balances, logs and checked assertions.
