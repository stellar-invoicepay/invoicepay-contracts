# invoicepay Contract — Error Codes

Every failure mode has a defined code. No silent fallback.

> Guardrail: no error codes are invented here. Every row matches one variant of
> `enum Error` in `src/types.rs`. Every variant has a test in
> `src/error_paths.rs` that triggers the real return path.
> `scripts/check-errors.mjs` runs in CI and fails if this file and the enum
> drift apart.

## Categories

| Range | Category |
|---|---|
| 1–9 | Lookup |
| 10–29 | Lifecycle & timing |
| 30–49 | Validation |

## Lookup (1–9)

| Code | Variant | Raised by | Trigger | User-facing message | Next action |
|---:|---|---|---|---|---|
| 1 | `InvoiceNotFound` | `pay`, `cancel`, `refund`, `get_invoice`, `receipt` | No invoice record exists for that id. | "We couldn't find that invoice. Check the invoice id with the freelancer." | Confirm the invoice id with the freelancer. |
| 2 | `PaymentNotFound` | `refund` | The freelancer tried to refund a payer who has not paid anything toward this invoice. | "That address has not paid anything on this invoice." | Check the payer address and the invoice. |

## Lifecycle & timing (10–29)

| Code | Variant | Raised by | Trigger | User-facing message | Next action |
|---:|---|---|---|---|---|
| 10 | `InvoiceCancelled` | `pay`, `cancel` | The freelancer has cancelled the invoice. | "This invoice has been cancelled and no longer accepts payments." | Ask the freelancer for a replacement invoice. |
| 11 | `InvoiceExpired` | `pay` | The invoice's due date (`due_at`) has passed. | "The due date for this invoice has passed." | Ask the freelancer whether a new invoice is coming; refunds still work. |
| 12 | `Overpayment` | `pay` | The payment would take the invoice past its total. | "That payment is more than the invoice still owes." | Pay the remaining amount or less; partial payments are allowed. |
| 13 | `ClientMismatch` | `pay` | The invoice is restricted to one client and the payer does not match. | "This invoice can only be paid by the client it was issued to." | Have the named client pay, or ask the freelancer to lift the restriction. |
| 14 | `CancelNotAllowed` | `cancel` | A payment has been made, so the invoice is a permanent receipt. | "This invoice has payments on it and cannot be cancelled." | Use `refund` to return funds instead. |
| 15 | `RefundTooLarge` | `refund` | The refund would exceed what that payer has paid, net of earlier refunds. | "That refund is more than this payer's remaining paid balance." | Refund an amount up to the payer's net paid total. |
| 16 | `PayersTooMany` | `pay` | The invoice already has the maximum number of distinct payers (250). | "This invoice cannot take payments from any more distinct payers." | Ask the freelancer to issue a new invoice. |

## Validation (30–49)

| Code | Variant | Raised by | Trigger | User-facing message | Next action |
|---:|---|---|---|---|---|
| 30 | `InvalidAmount` | `create_invoice`, `pay`, `refund` | The amount given was zero or negative. | "Amounts must be greater than zero." | Enter a positive amount. |
| 31 | `DueAtInPast` | `create_invoice` | The due date chosen is not in the future. | "The due date must be in the future." | Choose a new due date and recreate the invoice. |
