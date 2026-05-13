# mizan-smart

`mizan-smart` is the experimental **Premium Wealth OS + Local AI Intelligence**
branch of Mizan. It is cloned from the stable `mizan-4` repo and tracked
separately so the production branch is never destabilised by AI/web-evidence
work.

The full build plan for this branch lives at
[`docs/mizan-smart-plan/PLAN.md`](./mizan-smart-plan/PLAN.md). The plan is
divided into eight phases (Phase 0 through Phase 7) covering the senior-friendly
product shell, document-backed moat, serious-wealth coverage, Islamic/tax/reports,
web evidence, the local AI intelligence layer, and final entitlement +
hardening.

## Non-negotiable rules

These rules govern every change made in `mizan-smart`. They are the firewall
between Mizan's deterministic financial core and the new intelligence surfaces.

1. **The deterministic financial core remains the source of truth.** All money
   numbers are produced by Rust services using `rust_decimal`. No `f32`/`f64`
   in monetary calculations. Rounding only at the display/export boundary.
2. **AI suggestions cannot mutate financial truth.** AI may produce
   suggestions, explanations, classifications, summaries, query plans, draft
   commentary, and review hints. It may never write directly to the ledger,
   holdings, valuations, tax-pack lines, Shariah verdicts, approved extracted
   facts, source documents, or finalised report lines.
3. **Extracted facts require human approval.** Document extraction produces
   `pending` facts. Approval is an explicit user action and is audited. An
   approved fact is reviewed — it does not automatically post a ledger row.
4. **Web evidence requires review.** Web fetches produce evidence packs.
   No web fetch ever auto-updates a valuation. The user reviews each source
   and explicitly approves before a valuation row is written, and the written
   row carries provenance (URL, fetched_at, content hash).
5. **No paid APIs are core dependencies.** The product must work end-to-end
   without any paid search, market-data, AI, or compliance service. Optional
   provider plug-ins may exist but the default offline path must function.
6. **No cloud AI dependency for core functionality.** Local-first inference
   only (ONNX/GGUF/sidecar runtimes). If a runtime is missing, the feature
   exposes a disabled state with detection — never fakes success.
7. **Every AI output is structured, validated, auditable, and rejectable.**
   Outputs are persisted as `ai_suggestion_records`. Guardrail violations are
   logged. Uncited numeric claims are rejected.
8. **No fake rows, placeholder logic, or invented data.** Empty states are
   honest. If a parser, model, or runtime is unavailable the UI says so.

## Validation

Before any commit on this branch, the full validation suite must pass:

```
pnpm type-check
pnpm lint
CI=true pnpm --filter frontend exec vitest run
pnpm build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Relationship to the stable Mizan repo

`mizan-smart` is a separate GitHub repository
([samisayyed1/mizan-smart](https://github.com/samisayyed1/mizan-smart)) cloned
from [samisayyed1/mizan-4](https://github.com/samisayyed1/mizan-4) with full
history preserved. The stable branch continues independently. Selective
back-ports to stable happen by PR, never by force-push.
