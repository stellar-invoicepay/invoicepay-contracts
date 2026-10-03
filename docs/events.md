# Contract events

Only events the code actually emits are documented here. Anything not listed is
**not implemented yet**.

Events are defined with `#[contractevent]` in `src/types.rs` and published with
`EventName { .. }.publish(&env)`. The SDK puts the event's name into the topics
as a `Symbol`, followed by every field marked `#[topic]`; the remaining fields
land in the data map. All four events below are asserted with exact topic and
data layouts in `src/test.rs`.

## `InvoiceCreated`

Emitted once per invoice, by `create_invoice`.

| | |
|---|---|
| Topics | `Symbol("invoice_created")`, then `invoice_id` (`u64`) |
| Data | map with `freelancer` (`Address`), `token` (`Address`), `amount` (`i128`), `due_at` (`u64`), `details_hash` (`BytesN<32>`) |
| Emitted by | `create_invoice` in `src/invoices.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

The payer restriction (`client_opt`) is deliberately **not** in the event data:
it is stored with the invoice and readable via `get_invoice`.

## `InvoicePaid`

Emitted on every successful payment, by `pay`, after the tokens have moved
from the payer to the freelancer.

| | |
|---|---|
| Topics | `Symbol("invoice_paid")`, then `invoice_id` (`u64`) and `payer` (`Address`) |
| Data | map with `amount` (`i128`, this payment) and `total_paid` (`i128`, the invoice's `paid_total` after this payment) |
| Emitted by | `pay` in `src/invoices.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

An indexer can therefore follow one payer's payments across all invoices by
the topic pair alone.

## `InvoiceCancelled`

Emitted once per cancellation, by `cancel`.

| | |
|---|---|
| Topics | `Symbol("invoice_cancelled")`, then `invoice_id` (`u64`) |
| Data | map with `freelancer` (`Address`) |
| Emitted by | `cancel` in `src/invoices.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

## `InvoiceRefunded`

Emitted on every successful refund, by `refund`, after the tokens have moved
from the freelancer back to the payer.

| | |
|---|---|
| Topics | `Symbol("invoice_refunded")`, then `invoice_id` (`u64`) and `payer` (`Address`) |
| Data | map with `amount` (`i128`, this refund) and `total_refunded` (`i128`, the invoice's `refunded_total` after this refund) |
| Emitted by | `refund` in `src/invoices.rs` |
| Asserted in | `refund_publishes_the_documented_event` in `src/test.rs` |

## Not implemented yet

- No other events exist yet. New events are documented here in the same commit
  that adds them to `src/types.rs`.
