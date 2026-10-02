# Design decisions

One file per decision that a future contributor should not have to re-derive:
`NNNN-short-title.md`. Keep each one short — context, decision, consequences —
and dated.

The convention is a lightweight ADR:

```markdown
# NNNN. Short title

Date: YYYY-MM-DD
Status: accepted | superseded by NNNN

## Context
What forced a choice, in two or three sentences.

## Decision
What was chosen, precisely. Pin versions where a library is involved.

## Consequences
What this makes easy, what it makes hard, and what would change our mind.
```

## Recorded so far

- **0001 — No-custody token flow: direct transfer, no escrow, no extra
  crates** (*accepted*, 2026-10-03): the contract never holds funds;
  `pay` and `refund` move SEP-41 tokens directly between payer and
  freelancer, with no dependency beyond `soroban-sdk`.
