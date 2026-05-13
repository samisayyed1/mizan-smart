Yes. For **mizan-smart**, build the whole product in order: **Premium Wealth OS first, then Local AI Brain, then Web Evidence, then final hardening**. The docs all point to the same rule: Mizan must feel intelligent, but deterministic financial truth stays protected; AI creates suggestions/explanations only, and every material change requires review/approval.   The product moat is document-backed wealth truth, universal asset coverage, private investment/fixed-income support, tax packs, optional Islamic mode, and a simpler senior-friendly UX.     Web evidence must work as “public evidence, privately reviewed, never blindly trusted,” not as automatic price mutation.  

Below is the **full ordered Claude Code prompt pack**.

---

# **Global Prompt — Paste at the top of every Claude Code session**

```
You are working in the mizan-smart repository, cloned from the stable Mizan repo.

Read AGENTS.md and the entire codebase first. Inspect package.json, Cargo.toml workspace members, migrations, Tauri commands, frontend routing, existing tests, and current architecture before editing.

This repo is for building the full Mizan Premium Wealth OS plus Local AI Intelligence Layer.

Hard rules:
- Preserve all existing Mizan functionality.
- Do not remove working routes, commands, database fields, tests, services, imports, or UI flows unless explicitly replaced with a tested equivalent.
- Do not break existing portfolio math, imports, assets, dashboard, documents, reports, device sync, settings, or existing tests.
- No fake rows.
- No placeholder code.
- No invented data.
- No paid API dependency.
- No cloud AI dependency for core features.
- No direct AI mutation of ledger, holdings, valuations, reports, tax lines, Shariah verdicts, approved extracted facts, source documents, or portfolio truth.
- AI may only create suggestions, explanations, classifications, summaries, draft commentary, query plans, or review hints.
- Human approval is required before any AI/web/document output affects financial truth.
- Deterministic Rust services must calculate all financial numbers.
- Every AI output must be structured, validated, auditable, source-linked, and rejectable.
- Use rust_decimal or the repo’s existing Decimal strategy for all monetary calculations.
- Do not introduce f32/f64 into financial calculations.
- Keep TypeScript strict.
- No `any`.
- Add migrations for schema changes.
- Add backend tests and frontend tests for every feature.
- Keep existing tests passing.
- If an external binary/model/extension is not present, implement detection, settings, disabled states, and testable interfaces. Do not fake success.
- Commit only after validation is green.

Validation commands:
- pnpm typecheck
- pnpm lint
- pnpm test
- pnpm build
- cargo fmt
- cargo clippy -- -D warnings
- cargo test

If script names differ, inspect package.json/Cargo workspace and run the correct existing commands.
```

---

# **Phase 0 — Stable foundation**

## **Prompt 1 — Stabilize mizan-smart baseline**

```
Read AGENTS.md and the full codebase first.

Goal:
Create a clean stable baseline for the mizan-smart repo before building the intelligence layer.

Tasks:
1. Inspect:
   - git status
   - current branch
   - remotes
   - workspace structure
   - package.json scripts
   - Cargo workspace members
   - existing migrations
   - existing validation commands
2. Confirm this repo is separate from the stable main Mizan repo.
3. Run:
   - pnpm typecheck
   - pnpm lint
   - pnpm test
   - pnpm build
   - cargo fmt
   - cargo clippy -- -D warnings
   - cargo test
4. Fix only real validation failures.
5. Do not add product features in this prompt.
6. Add `docs/mizan-smart-branch.md` documenting:
   - mizan-smart is the experimental Premium Wealth OS + Local AI Intelligence branch
   - deterministic financial core remains the source of truth
   - AI suggestions cannot mutate financial truth
   - extracted facts require human approval
   - web evidence requires review
   - no paid APIs are core dependencies
7. Commit after all checks pass.

Do not weaken tests.
Do not skip checks.
Do not introduce new dependencies unless needed for validation repair.
```

---

# **Phase 1 — Senior-friendly product shell**

## **Prompt 2 — Boomer-friendly navigation and UI simplification**

```
Read AGENTS.md and the full codebase first.

Goal:
Simplify the app into a premium, senior-friendly wealth OS without deleting existing functionality.

Primary navigation:
- Home
- Portfolio
- Documents
- Reports
- Inbox
- Settings

Rules:
1. Do not remove existing routes. Move advanced pages behind secondary navigation.
2. Keep all old pages accessible through advanced links, redirects, or Settings.
3. Hide noisy developer/trading-style screens from primary navigation unless essential.
4. Replace icon-only primary navigation/actions with icon + plain-English text.
5. Increase readability:
   - base UI text at least 16px where practical
   - larger section headings
   - stronger contrast
   - calm premium spacing
6. Remove decorative gradients/motion where they distract from financial clarity.
7. Keep mobile/tablet layout functional.

Frontend work:
- Update main layout/sidebar/nav components.
- Add simple section grouping.
- Add empty states with clear copy:
  - “Add your first asset”
  - “Upload a statement”
  - “Review what needs attention”
- Add senior-friendly labels:
  - “What changed”
  - “Needs attention”
  - “Income received”
  - “Documents to review”

Tests:
- Main navigation renders exact primary sections.
- Existing routes still resolve.
- Hidden advanced pages remain reachable.
- No route crashes.
- Accessibility smoke test if test tooling exists.

Run full validation and commit only when green.
```

---

## **Prompt 3 — Home dashboard command center**

```
Read AGENTS.md and the full codebase first.

Goal:
Rebuild Home into a useful command center instead of an empty dashboard.

Create dashboard modules using real data only:

1. Net Worth Summary
   - total net worth
   - month/period change if available
   - base currency
   - last updated
   - link to Explain This Number later
   - honest empty state if no assets

2. What Changed
   - largest value changes if data exists
   - income received this month
   - new transactions
   - stale/missing data warnings
   - no fake movers

3. Wealth Inbox Preview
   - active alerts
   - pending document reviews if tables exist
   - stale valuations
   - missing FX
   - upcoming capital calls/coupons once implemented
   - empty state: “Nothing needs attention”

4. Income This Month
   - dividends
   - interest
   - coupons/profit if available
   - rental/private distributions later
   - data-source warning if incomplete

5. Data Quality Preview
   - placeholder only if data-quality service not implemented yet
   - no fake score
   - show “Data quality will appear after checks run”

6. Quick Actions
   - Add Asset
   - Update Values
   - Upload Document
   - Review Inbox
   - Generate Report

Rules:
- No fake numbers.
- No sample/demo values.
- Use TanStack Query if repo pattern supports it.
- All money formatting must use existing currency formatting helpers.
- Use real loading/error/empty states.

Tests:
- empty portfolio state
- populated portfolio state with fixtures/mocks
- quick actions link correctly
- module error state does not crash dashboard

Run validation and commit only when green.
```

---

## **Prompt 4 — Universal asset model foundation**

```
Read AGENTS.md and the full codebase first.

Goal:
Prepare Mizan to track the full financial life of serious investors.

Asset classes to support:
- public_equity
- etf
- mutual_fund
- fixed_income
- sukuk
- fixed_deposit
- cash
- real_estate
- private_equity
- private_credit
- hedge_fund
- venture_capital
- crypto
- commodity
- gold
- silver
- insurance
- ulip
- pension
- business_ownership
- collectible
- liability
- custom

Schema approach:
Use base asset + typed extension tables. Do not use messy EAV.

Tasks:
1. Preserve existing assets table compatibility.
2. Add/extend asset_type enum safely.
3. Add typed tables only where needed:
   - asset_public_market_details
   - asset_fixed_income_details
   - asset_real_estate_details
   - asset_private_investment_details
   - asset_insurance_details
   - asset_commodity_details
   - asset_business_details
   - asset_collectible_details
   - asset_liability_details
4. Add valuations table if missing:
   - id
   - asset_id
   - valuation_date
   - value_native
   - currency
   - source_type: manual | market | document | import | web_evidence | calculated
   - source_id nullable
   - confidence nullable
   - notes nullable
   - created_at
   - updated_at
5. Add indexes:
   - asset_id + valuation_date
   - asset_type
   - currency
6. Add migrations with FK constraints.
7. Ensure old assets migrate safely.
8. Existing public equity/portfolio calculations must continue to work.

Backend:
- Add typed domain structs.
- Add conversion/mapping logic.
- Add CRUD where needed.
- For unsupported detailed valuation engines, store as manually valued with clear status.

Tests:
- migration works
- create each supported asset type
- invalid subtype data rejected
- valuation insert/list works
- existing portfolio tests still pass

Run validation and commit only when green.
```

---

## **Prompt 5 — Universal Add Asset redesign**

```
Read AGENTS.md and the full codebase first.

Goal:
Replace complex asset creation with a simple senior-friendly universal flow.

Flow:
1. “What are you adding?”
   Cards:
   - Stock / ETF / Fund
   - Bond / Sukuk
   - Fixed Deposit / Cash
   - Property
   - Private Investment
   - Gold / Commodity
   - Crypto
   - Insurance / ULIP
   - Business / Other
   - Liability

2. Required fields only.
3. Optional advanced details.
4. Review and save.

Frontend:
- Use React Hook Form + Zod.
- Create discriminated union schemas per asset type.
- Keep one simple route for Add Asset.
- Use plain labels:
  - “Current value”
  - “When was this value last checked?”
  - “Maturity date”
  - “Expected payment”
  - “Upload document later”
- Add “I don’t know yet” where optional fields can be omitted safely.
- No overwhelming 8-step wizard.

Backend:
- Reuse existing commands where possible.
- Add new command only if needed:
  - create_universal_asset
- Validate asset type and subtype payloads.
- No fake lookups.
- If symbol lookup fails, allow manual creation.

Tests:
- each card renders correct fields
- required validation works
- optional fields omitted safely
- save creates correct base + typed records
- existing asset creation flow still works or redirects safely

Run validation and commit only when green.
```

---

## **Prompt 6 — Manual valuations and bulk update grid**

```
Read AGENTS.md and the full codebase first.

Goal:
Make it easy for older/HNW users to update all manually valued assets.

Build “Update Values” screen.

Features:
1. List all manually valued assets:
   - property
   - private investments
   - gold/commodities
   - business ownership
   - insurance/ULIP
   - collectibles
   - custom assets
2. Editable columns:
   - current value
   - valuation date
   - currency
   - notes
3. One-click “Mark unchanged”
4. Batch save in one SQLite transaction
5. Stale valuation rules:
   - older than 45 days = warning
   - older than 90 days = critical
6. Link:
   - Upload source document
   - Find web evidence later
   - View history

Backend:
- Tauri command: bulk_update_valuations
- Validate all rows before writing.
- Decimal-only money parsing.
- Return structured row-level validation errors.
- If any row is invalid, do not partially write unless existing architecture explicitly supports partial imports.

Frontend:
- TanStack Table
- senior-friendly row height
- sticky save bar
- clear validation messages
- no tiny dense grid

Tests:
- valid batch saves
- invalid batch rolls back
- stale indicators render
- Decimal parsing rejects invalid strings
- valuation history is preserved

Run validation and commit only when green.
```

---

## **Prompt 7 — Data Quality Score**

```
Read AGENTS.md and the full codebase first.

Goal:
Create a professional data-quality layer that makes Mizan trustworthy.

Backend service:
calculate_data_quality()

Score components:
- stale manual valuations
- stale market quotes
- missing FX
- unclassified assets
- missing current valuation
- pending health issues
- pending document review once Document Vault exists
- uncited report lines once reports exist
- missing fixed-income terms once fixed-income exists
- missing private fund NAV once private investments exist

Return:
- score out of 100
- status: excellent | good | needs_attention | poor
- deductions array:
  - category
  - points
  - explanation
  - severity
  - action_route
  - source_entity_type
  - source_entity_id

Frontend:
- Dashboard card
- Data Quality detail page
- Calm professional UI, not gamified
- Click each issue to fix

Rules:
- no fake score if portfolio empty
- empty portfolio shows onboarding state
- do not use aggressive red unless critical
- all deductions deterministic

Tests:
- empty portfolio
- perfect data
- stale valuation deductions
- missing FX deductions
- multiple deductions sum correctly
- action routes valid

Run validation and commit only when green.
```

---

## **Prompt 8 — Smart alert engine**

```
Read AGENTS.md and the full codebase first.

Goal:
Implement deterministic local alerts.

Schema:
smart_alerts:
- id
- fingerprint unique
- rule_name
- category
- severity: info | warning | critical
- title
- message
- status: active | snoozed | dismissed | resolved
- source_entity_type nullable
- source_entity_id nullable
- action_route nullable
- first_seen_at
- last_seen_at
- snoozed_until nullable
- dismissed_at nullable
- resolved_at nullable
- metadata_json nullable

Rules to implement:
1. StaleManualValuationRule
2. StaleMarketQuoteRule
3. MissingFXRule
4. UnclassifiedAssetRule
5. MissingCurrentValuationRule
6. HighConcentrationRule
7. PendingDocumentReviewRule if document tables exist
8. MissingPrivateFundNavRule later-compatible
9. FixedIncomeTermsIncompleteRule later-compatible

Backend:
- rule trait/interface
- run_all_alert_rules command
- list_alerts command
- snooze_alert command
- dismiss_alert command
- resolve_alert command
- dedupe via fingerprint
- no alert spam

Frontend:
- alert list
- dashboard preview
- fix button
- snooze/dismiss
- plain-English wording

Tests:
- each rule triggers
- repeated runs update last_seen_at not duplicates
- snooze hides until due
- dismiss works
- resolved when condition fixed

Run validation and commit only when green.
```

---

## **Prompt 9 — Wealth Inbox**

```
Read AGENTS.md and the full codebase first.

Goal:
Build Wealth Inbox as the central action center.

Inbox aggregates:
- active alerts
- stale valuation tasks
- missing FX tasks
- unclassified asset tasks
- pending document reviews once available
- upcoming capital calls once available
- upcoming fixed-income payments once available
- tax pack missing items once available
- memory suggestions once AI memory exists
- web evidence reviews once web evidence exists

Create normalized inbox view model:
- id
- type
- title
- description
- severity
- due_date nullable
- source_entity_type
- source_entity_id
- action_route
- status
- created_at

Frontend:
- Inbox page
- filters:
  - All
  - Documents
  - Valuations
  - Tax
  - Income
  - Private Investments
  - Security
  - AI Suggestions
  - Web Evidence
- sort:
  - critical first
  - due soon
  - newest
- one-click action routing

Rules:
- no fake tasks
- empty state says “Nothing needs attention”
- every item must have a real source or deterministic rule

Tests:
- alerts appear
- stale valuations appear
- sorting works
- filtering works
- action routes valid
- empty state works

Run validation and commit only when green.
```

---

# **Phase 2 — Document-backed moat**

## **Prompt 10 — Document Vault encrypted storage**

```
Read AGENTS.md and the full codebase first.

Goal:
Create secure local Document Vault storage.

Schema:
documents:
- id
- file_hash unique
- original_name
- mime_type
- file_size_bytes
- encrypted_storage_path
- status: ingested | queued | processing | processed | reviewed | error
- source_type nullable
- error_message nullable
- created_at
- updated_at

document_files:
- id
- document_id
- encryption_version
- nonce
- checksum_sha256
- storage_path
- created_at

document_links:
- id
- document_id
- linked_entity_type
- linked_entity_id
- created_at

Backend:
- upload_document command
- list_documents command
- delete_document command
- get_document_metadata command
- compute SHA-256
- reject exact duplicates
- encrypt file using existing crypto/keyring pattern
- store encrypted file in app data directory
- decrypt only through explicit read path
- never leave decrypted temp files behind

Frontend:
- Documents page
- Upload button/dropzone
- document list
- status
- duplicate error state
- delete action

Tests:
- duplicate rejected
- encrypted file exists
- decrypt round-trip
- delete removes encrypted file
- metadata persisted
- upload error handled

Run validation and commit only when green.
```

---

## **Prompt 11 — Document processing job system**

```
Read AGENTS.md and the full codebase first.

Goal:
Create robust background document processing infrastructure.

Schema:
document_processing_jobs:
- id
- document_id
- job_type: parse_text | extract_layout | extract_tables | ocr | vlm_extract | embed
- status: queued | running | succeeded | failed | cancelled
- priority
- attempts
- max_attempts
- error_message nullable
- started_at nullable
- completed_at nullable
- created_at

Backend:
- enqueue_document_job
- list_document_jobs
- run_next_document_job
- cancel_document_job
- retry failed job
- background worker lifecycle safe for Tauri
- no blocking main thread
- timeout support
- structured errors

Rules:
- if parser not available, job fails honestly
- no fake parsed output
- failed jobs visible in UI

Frontend:
- processing status on Documents page
- retry failed job
- pause/resume processing if practical

Tests:
- upload enqueues job
- worker runs queued job
- failure stored
- retry limit enforced
- cancellation works

Run validation and commit only when green.
```

---

## **Prompt 12 — Local PDF/text/layout extraction adapter**

```
Read AGENTS.md and the full codebase first.

Goal:
Add a real local extraction adapter with safe capability detection.

Research existing repo constraints first. Implement an adapter interface, not hardcoded fragile logic.

Backend abstraction:
DocumentParser:
- capabilities()
- parse_document(document_id) -> ParsedDocument
- parse_text
- parse_layout
- parse_tables if supported
- returns page numbers, text blocks, bounding boxes, confidence where available

Supported runtime strategy:
1. If a Rust PDF parser exists or can be added safely, use it for text extraction.
2. If external parser/sidecar is required, create configurable sidecar wrapper.
3. If MinerU/Marker/Ferrules is unavailable, expose “parser unavailable” status and keep app stable.

Schema:
document_pages:
- id
- document_id
- page_number
- width nullable
- height nullable
- rotation nullable
- created_at

document_text_blocks:
- id
- document_id
- page_number
- text
- bounding_box_json nullable
- block_order
- confidence nullable
- created_at

document_tables:
- id
- document_id
- page_number
- bounding_box_json nullable
- created_at

document_table_cells:
- id
- table_id
- row_index
- column_index
- text
- bounding_box_json nullable
- confidence nullable

Rules:
- no cloud extraction
- no fake extraction
- never block UI
- temp files cleaned

Tests:
- fixture PDF parses text if parser available
- unsupported parser returns clear error
- corrupted PDF fails cleanly
- temp cleanup verified where possible

Run validation and commit only when green.
```

---

## **Prompt 13 — Extracted facts and citations**

```
Read AGENTS.md and the full codebase first.

Goal:
Create citation-backed extracted facts.

Schema:
extracted_facts:
- id
- document_id
- page_number nullable
- fact_type
- raw_value
- normalized_value nullable
- currency nullable
- date_value nullable
- confidence_score nullable
- bounding_box_json nullable
- extraction_method: parser | ocr | vlm | manual
- extraction_version
- status: pending | approved | rejected | superseded
- created_at
- reviewed_at nullable
- review_notes nullable

source_citations:
- id
- source_type: document | manual | import | web_evidence | calculated
- source_id nullable
- document_id nullable
- extracted_fact_id nullable
- page_number nullable
- bounding_box_json nullable
- citation_label
- created_at

Add nullable citation links where safe:
- valuations.source_citation_id
- activities/transactions source_citation_id if existing schema supports
- private investment rows later
- report lines later

Backend:
- create_extracted_fact
- list_pending_extracted_facts
- get_source_citation
- approve/reject status updates only, not ledger writes yet

Rules:
- extracted facts cannot update ledger automatically
- approved fact means fact reviewed, not necessarily posted
- no fake confidence

Tests:
- FK constraints
- pending/approved/rejected transitions
- cannot approve deleted document
- citation lookup works
- rejected facts remain auditable

Run validation and commit only when green.
```

---

## **Prompt 14 — Document Review Queue**

```
Read AGENTS.md and the full codebase first.

Goal:
Build human-in-the-loop extracted fact review.

Frontend:
- Documents > Review Queue
- split screen:
  - left: document/page viewer or text block viewer if PDF renderer unavailable
  - right: extracted facts
- show:
  - raw value
  - normalized value
  - confidence
  - page number
  - source highlight/bounding box if available
  - suggested target mapping if available
- actions:
  - approve fact
  - edit and approve
  - reject
  - link to asset/account
  - defer

Backend:
- approve_extracted_fact
- reject_extracted_fact
- update_extracted_fact_before_approval
- link_extracted_fact_to_entity
- transaction-safe updates

Rules:
- approving fact does not automatically update ledger unless user explicitly chooses “Create valuation/activity from this fact”
- no silent writes
- no auto approval
- all edits logged

Tests:
- pending facts render
- approve changes status
- reject changes status
- edit validates normalized value
- link to asset/account works
- no ledger row created accidentally

Run validation and commit only when green.
```

---

## **Prompt 15 — Explain This Number**

```
Read AGENTS.md and the full codebase first.

Goal:
Make major numbers traceable and trusted.

Backend command:
get_data_lineage(entity_type, entity_id, metric_type)

Support:
- net_worth
- asset_value
- valuation
- income_this_month
- data_quality_score
- alert_reason
- private_investment_metric later
- tax_pack_line later
- zakat_line later

Lineage response:
- displayed_value
- currency
- formula_name
- formula_description
- input_rows
- source_citations
- source_documents
- FX rates used
- valuation dates
- rounding policy
- stale/missing data warnings
- confidence/freshness
- last_updated

Frontend:
- ExplainableNumber component
- modal:
  - formula
  - inputs
  - citations
  - warnings
  - “No source document linked yet” if missing
  - no AI wording yet

Rules:
- never invent lineage
- if citation missing, say so
- deterministic only

Tests:
- net worth lineage
- valuation lineage
- missing citation path
- stale warning path
- modal renders source links

Run validation and commit only when green.
```

---

## **Prompt 16 — Reconciliation Center**

```
Read AGENTS.md and the full codebase first.

Goal:
Allow users to prove Mizan against statements/imports.

Schema:
reconciliation_runs:
- id
- scope_type: account | asset | document | import
- scope_id
- status
- created_at
- completed_at nullable

reconciliation_items:
- id
- run_id
- item_type
- source_side: mizan | external
- raw_json
- normalized_hash
- amount nullable
- currency nullable
- effective_date nullable
- status

reconciliation_matches:
- id
- run_id
- mizan_item_id nullable
- external_item_id nullable
- match_status: matched | possible_match | missing_in_mizan | missing_in_external | duplicate | mismatch
- confidence
- reason
- created_at

Backend:
- reconcile_account
- reconcile_document_facts
- reconcile_import_preview
- exact amount/date/currency matching
- date tolerance configurable
- Decimal comparisons only
- no AI required

Frontend:
- Reconciliation Center
- side-by-side rows
- accept adjustment
- ignore with reason
- manual match

Rules:
- no auto-write
- accepted adjustment requires user click
- accepted adjustment carries citation/source when available

Tests:
- exact match
- date tolerance
- duplicate detection
- missing in Mizan
- mismatch
- accept adjustment writes exactly one row

Run validation and commit only when green.
```

---

# **Phase 3 — Serious wealth coverage**

## **Prompt 17 — Private investments foundation**

```
Read AGENTS.md and the full codebase first.

Goal:
Add first-class private investments.

Schema:
private_investments:
- asset_id
- manager
- strategy
- vintage_year nullable
- commitment_amount
- commitment_currency
- inception_date nullable
- notes nullable

private_investment_valuations:
- id
- asset_id
- valuation_date
- nav
- currency
- source_citation_id nullable

capital_calls:
- id
- asset_id
- notice_date
- due_date
- amount
- currency
- status: expected | due | paid | cancelled
- source_citation_id nullable
- notes nullable

private_distributions:
- id
- asset_id
- distribution_date
- amount
- currency
- recallable
- source_citation_id nullable
- notes nullable

Backend:
- CRUD commands
- metrics:
  - commitment
  - paid_in_capital
  - unfunded_commitment
  - total_distributions
  - current_nav
  - DPI
  - RVPI
  - TVPI
  - MOIC
- Decimal only

Rules:
- no fake NAV
- distributions cannot silently exceed logical constraints; warn/flag
- document citations optional but supported

Tests:
- commitment math
- capital call paid-in
- unfunded commitment
- recallable distribution behavior
- DPI/RVPI/TVPI/MOIC examples

Run validation and commit only when green.
```

---

## **Prompt 18 — Private investment UI and J-curve**

```
Read AGENTS.md and the full codebase first.

Goal:
Make private investments visible and premium.

Frontend:
- Private Investment detail page
- show:
  - commitment
  - paid-in
  - unfunded
  - NAV
  - distributions
  - DPI
  - RVPI
  - TVPI
  - MOIC
  - upcoming capital calls
  - linked documents
- forms:
  - add capital call
  - mark call paid
  - add distribution
  - update NAV
- J-curve chart:
  - cumulative net cashflow
  - NAV overlay if available

Backend:
- detail query optimized for one private investment
- all numbers from deterministic metrics service

Rules:
- no fake chart data
- empty states guide user
- every citation shown if present
- no investment advice

Tests:
- empty private fund renders
- populated metrics render
- add capital call updates metrics
- chart handles no data
- mark paid flow works

Run validation and commit only when green.
```

---

## **Prompt 19 — Fixed income / Sukuk / Fixed Deposit engine**

```
Read AGENTS.md and the full codebase first.

Goal:
Add serious fixed-income tracking.

Schema:
asset_fixed_income_details:
- asset_id
- instrument_type: bond | sukuk | treasury_bill | fixed_deposit | cd | structured_note | other
- issuer
- isin nullable
- face_value
- currency
- purchase_date nullable
- maturity_date
- coupon_or_profit_rate nullable
- payment_frequency nullable
- day_count_convention: ACT_360 | ACT_365 | ACT_ACT | THIRTY_360
- is_sukuk
- source_citation_id nullable

fixed_income_cashflows:
- id
- asset_id
- expected_date
- cashflow_type: coupon | profit | principal | maturity | interest
- expected_amount
- actual_amount nullable
- currency
- status: expected | received | missed | cancelled
- source_citation_id nullable

Backend:
- day-count functions
- accrued interest/profit calculation
- projected cashflow schedule generation
- fixed deposit maturity schedule
- Sukuk label uses “profit” not “interest”
- no price feed dependency

Tests:
- ACT/360
- ACT/365
- 30/360
- ACT/ACT if practical
- coupon/profit schedule
- fixed deposit maturity
- incomplete setup warning

Run validation and commit only when green.
```

---

## **Prompt 20 — Liquidity Ladder**

```
Read AGENTS.md and the full codebase first.

Goal:
Give retirees and HNW users a clear cashflow timeline.

Build Liquidity Ladder.

Inputs:
- cash balances
- fixed income cashflows
- Sukuk profit payments
- fixed deposit maturities
- private capital calls
- private distributions
- dividends/interest where scheduled or historically known
- tax obligations later
- insurance premiums later

Views:
- next 30 days
- next 90 days
- next 12 months
- group by currency
- incoming vs outgoing
- confirmed vs expected

Frontend:
- dashboard card
- detailed report/table
- timeline visualization
- plain-English copy

Rules:
- do not invent future dividends
- expected vs confirmed clearly labeled
- missing data shown honestly

Tests:
- fixed income cashflows included
- capital calls included
- currency grouping
- empty state
- expected/confirmed labels

Run validation and commit only when green.
```

---

## **Prompt 21 — Corporate actions engine**

```
Read AGENTS.md and the full codebase first.

Goal:
Harden public-market accounting.

Schema:
corporate_actions:
- id
- asset_id
- action_type: split | reverse_split | merger | spinoff | symbol_change | return_of_capital | stock_dividend
- effective_date
- ratio_numerator nullable
- ratio_denominator nullable
- new_symbol nullable
- metadata_json nullable
- source_citation_id nullable
- created_at

Implement first:
- split
- reverse_split
- symbol_change

Backend:
- apply_stock_split
- adjust historical lots deterministically
- preserve total cost basis for splits
- transaction-safe
- immutable audit event

Frontend:
- Corporate Actions screen under asset detail
- add action
- preview effect before applying
- confirm

Rules:
- user initiated/reviewed only
- no web auto-apply
- Decimal only

Tests:
- 2:1 split
- reverse split
- cost basis preserved
- quantity adjusted
- invalid ratio rejected
- symbol change preserves history

Run validation and commit only when green.
```

---

## **Prompt 22 — Accuracy invariant hardening**

```
Read AGENTS.md and the full codebase first.

Goal:
Make Mizan’s financial core provably reliable.

Audit:
- financial calculations
- imports
- valuations
- private investments
- fixed income
- reports
- exports

Rules:
- no f32/f64 in money paths
- Decimal/rust_decimal only
- rounding only at display/export boundary
- full precision internally
- missing FX fails explicitly
- no silent fallback unless labeled

Add invariant tests:
1. Sum of open lots equals holding quantity.
2. Sum of lot cost basis equals holding cost basis.
3. Realized gain = proceeds - cost basis - fees.
4. Cash ledger equals cash balance where applicable.
5. Split preserves total cost basis.
6. Report totals equal line sums.
7. Private investment paid-in/unfunded invariant.
8. Fixed-income cashflow totals match schedule.

Use proptest where practical.
Use golden files for core scenarios.

Run validation and commit only when green.
```

---

## **Prompt 23 — Golden import templates**

```
Read AGENTS.md and the full codebase first.

Goal:
Make imports deterministic and safe.

Implement deterministic templates:
- Yahoo Finance holdings CSV
- Yahoo Finance transactions CSV if applicable
- IBKR activity CSV
- Fidelity CSV
- Schwab CSV
- generic bank CSV
- fixed deposit CSV template
- private investment capital call CSV template
- fixed income cashflow CSV template

Rules:
- strict header matching
- no AI mapping for golden templates
- unknown columns produce clear preview warning
- required fields enforced
- dry-run preview required
- duplicate detection preserved
- no fake valid rows
- invalid rows cannot silently pass

Tests:
- fixture for each template
- bad header rejected
- duplicate row detected
- missing field rejected
- date/currency parsing
- no partial invalid import unless explicitly reviewed

Run validation and commit only when green.
```

---

# **Phase 4 — Islamic mode, tax packs, reports**

## **Prompt 24 — Optional Islamic mode foundation**

```
Read AGENTS.md and the full codebase first.

Goal:
Add Islamic finance as an optional overlay, not the identity of the app.

Setting:
- shariah_mode_enabled default false

Schema:
shariah_screening_profiles:
- id
- name
- debt_threshold
- liquid_assets_threshold
- impure_income_threshold
- is_default
- created_at
- updated_at

asset_shariah_screening:
- id
- asset_id
- profile_id
- status: compliant | non_compliant | questionable | unknown | needs_review
- debt_ratio nullable
- liquid_assets_ratio nullable
- impure_income_ratio nullable
- source_citation_id nullable
- manual_override_reason nullable
- reviewed_at nullable
- created_at
- updated_at

Default thresholds:
- debt < 30%
- liquid assets < 30%
- impure income < 5%

Frontend:
- Settings toggle
- when disabled: no Shariah/zakat/purification UI visible
- when enabled: asset status badges, screening page, zakat section

Rules:
- no forced Islamic framing
- no paid compliance API
- no official fatwa/certification claims
- insufficient data = unknown/needs_review

Tests:
- disabled hides UI
- enabled shows UI
- thresholds evaluate
- missing ratios produce unknown
- existing non-Islamic users unaffected

Run validation and commit only when green.
```

---

## **Prompt 25 — Shariah screening workflow**

```
Read AGENTS.md and the full codebase first.

Goal:
Make screening auditable and reviewable.

Frontend:
- Screening Profiles page
- Asset-level screening form:
  - debt ratio
  - liquid assets ratio
  - impure income ratio
  - source citation
  - notes
  - manual override reason
- Review status
- history/audit display if audit exists

Backend:
- evaluate_shariah_compliance(asset_id, profile_id)
- create/update screening result
- manual override requires reason
- source citation optional but supported

Rules:
- user-entered ratios clearly labeled
- document-backed ratios show citation
- no paid screening data
- no final religious/legal advice

Tests:
- pass case
- fail debt case
- fail liquid assets case
- fail impure income case
- manual override without reason rejected
- disabled mode blocks feature

Run validation and commit only when green.
```

---

## **Prompt 26 — Zakat calculator**

```
Read AGENTS.md and the full codebase first.

Goal:
Implement optional zakat workflow.

Schema:
zakat_snapshots:
- id
- snapshot_date
- base_currency
- total_zakatable_assets
- deductible_liabilities
- net_zakatable_wealth
- nisab_value
- zakat_due
- notes
- created_at

zakat_lines:
- id
- snapshot_id
- asset_id nullable
- category
- amount
- included
- explanation
- source_citation_id nullable

Backend:
- calculate_zakat_snapshot
- manual nisab input first
- future gold/silver evidence may feed suggested nisab, but user confirms
- Decimal only
- lineage for each zakat line

Frontend:
- guided wizard
- select assets/categories
- enter nisab
- review included/excluded lines
- final summary
- export if report infrastructure exists

Rules:
- Islamic mode only
- no religious advice claim
- no fake nisab
- user controls inclusion

Tests:
- short-term asset included at market value
- liability deduction
- manual nisab
- line-level explanations
- disabled mode blocked

Run validation and commit only when green.
```

---

## **Prompt 27 — Dividend purification calculator**

```
Read AGENTS.md and the full codebase first.

Goal:
Track optional purification amounts.

Schema:
purification_entries:
- id
- asset_id
- period_start
- period_end
- total_impure_income nullable
- outstanding_shares nullable
- user_shares nullable
- dividend_received nullable
- impure_income_ratio nullable
- purification_amount
- calculation_method
- status: calculated | paid | waived
- source_citation_id nullable
- notes nullable
- created_at
- updated_at

Backend:
Methods:
1. impure income per share:
   (total_impure_income / outstanding_shares) * user_shares
2. dividend ratio:
   dividend_received * impure_income_ratio

Frontend:
- purification table
- add/review entry
- mark paid
- export summary

Rules:
- Islamic mode only
- insufficient data = needs review
- no fake ratios
- Decimal only

Tests:
- both calculation methods
- missing data path
- mark paid
- totals by period

Run validation and commit only when green.
```

---

## **Prompt 28 — Tax pack foundation**

```
Read AGENTS.md and the full codebase first.

Goal:
Create CPA-ready data preparation packs.

Schema:
tax_packs:
- id
- tax_year
- jurisdiction: US | UK | Singapore | GCC | General
- base_currency
- status: draft | finalized | exported
- created_at
- finalized_at nullable

tax_pack_lines:
- id
- tax_pack_id
- category: realized_gain | dividend | interest | coupon | fx | private_distribution | fee | other
- asset_id nullable
- activity_id nullable
- amount
- currency
- taxable_date
- source_citation_id nullable
- notes nullable

Backend:
- generate_tax_pack(tax_year, jurisdiction)
- pull realized gains from existing FIFO logic
- dividends/interest/coupons
- private distributions if present
- FX notes where available
- missing data checklist

Rules:
- data preparation only, not tax advice
- no fake tax classification
- every line traces to ledger/citation/manual source where available
- no jurisdiction-specific filing claims beyond summaries

Tests:
- tax year filtering
- realized gain lines
- dividend line
- coupon line
- missing citation warning
- empty draft with checklist

Run validation and commit only when green.
```

---

## **Prompt 29 — CPA export bundle**

```
Read AGENTS.md and the full codebase first.

Goal:
Export tax packs for accountants.

Export bundle:
- ZIP
- summary PDF or HTML/PDF depending on existing stack
- CSV/XLSX line-item export
- source document manifest
- source_documents folder for linked source docs where available
- disclaimer

Backend:
- generate_tax_pack_export(tax_pack_id)
- no decrypted document leaks
- deterministic file naming
- manifest includes missing docs

Frontend:
- export button
- generated file download/save
- export history if existing report infra supports

Rules:
- no tax advice
- no fake source documents
- Decimal precision preserved
- source docs only included if user approves/feature allows

Tests:
- ZIP contains expected files
- CSV exact values
- disclaimer present
- missing source flagged
- no temp leak if testable

Run validation and commit only when green.
```

---

## **Prompt 30 — Report Builder foundation**

```
Read AGENTS.md and the full codebase first.

Goal:
Create reusable deterministic report infrastructure.

Schema:
report_runs:
- id
- report_type
- base_currency
- status
- created_at
- completed_at nullable

report_sections:
- id
- report_run_id
- title
- section_order
- metadata_json nullable

report_lines:
- id
- section_id
- label
- amount nullable
- currency nullable
- value_text nullable
- source_citation_id nullable
- metadata_json nullable

Reports:
- Net Worth Report
- Portfolio Summary
- Income Report
- Data Quality Report
- Tax Pack Report
- Private Investment Report later
- Zakat Report later

Rules:
- deterministic reports only
- no AI commentary yet
- line citations where available
- missing citation clearly labeled

Frontend:
- Reports page
- select report type
- preview
- export

Tests:
- report run created
- lines generated
- citations included
- empty report state
- export bytes generated if export implemented

Run validation and commit only when green.
```

---

## **Prompt 31 — Monthly Wealth Letter deterministic version**

```
Read AGENTS.md and the full codebase first.

Goal:
Create a premium monthly summary without LLM dependency first.

Sections:
- opening summary
- net worth change
- income received
- largest contributors if available
- fees if available
- stale/missing data
- pending document reviews
- upcoming capital calls
- upcoming coupons/maturities
- tax readiness
- zakat readiness if enabled
- data quality score

Backend:
- deterministic template engine
- every number from report lines or portfolio services
- no AI yet

Frontend:
- Reports > Monthly Wealth Letter
- preview
- export

Rules:
- no unsupported sections if no data
- no fake commentary
- no advice

Tests:
- month with data
- empty month
- exact values preserved
- missing sections omitted

Run validation and commit only when green.
```

---

## **Prompt 32 — Estate / Legacy Binder**

```
Read AGENTS.md and the full codebase first.

Goal:
Create premium older-user value without legal advice.

Estate Binder contents:
- accounts
- assets
- liabilities
- property
- insurance/ULIP
- pensions
- private investments
- key contacts notes
- documents manifest
- entity ownership summary if available
- optional zakat/waqf/charity notes if Islamic mode enabled

Frontend:
- Reports > Estate Binder
- choose included sections
- preview
- export encrypted archive if existing encryption supports it

Rules:
- explicitly not legal advice
- no will/trust generation
- no fake contacts
- user chooses sections
- citations where available

Tests:
- section selection
- export contains selected sections only
- disclaimer present
- encrypted archive round-trip if supported

Run validation and commit only when green.
```

---

## **Prompt 33 — Fee Intelligence**

```
Read AGENTS.md and the full codebase first.

Goal:
Help users understand fees.

Fee categories:
- broker fees
- transaction fees
- platform fees
- advisory fees
- fund expense ratio manual
- insurance/ULIP charges
- FX fees
- private fund fees
- custody/admin fees
- other

Backend:
- classify existing fee fields where present
- allow manual fee entries
- aggregate by period/account/asset/category
- detect fee spike deterministically

Frontend:
- Reports > Fee Report
- dashboard fee warning if spike
- Explain This Number support

Rules:
- no hidden fee claims without source
- extracted fees require review/citation
- no advice

Tests:
- fee aggregation
- period filter
- fee spike alert
- report export
- missing fees state

Run validation and commit only when green.
```

---

## **Prompt 34 — Concentration and fragility radar**

```
Read AGENTS.md and the full codebase first.

Goal:
Show concentration risks plainly without giving advice.

Compute concentration across:
- asset
- account/custodian
- currency
- sector/taxonomy
- country/taxonomy
- asset class
- income source
- manually valued/stale exposure
- private/illiquid exposure
- Shariah unknown exposure if enabled
- document-uncited exposure if citations exist

Frontend:
- Dashboard card
- Report section
- plain English:
  - “42% of income comes from two assets.”
  - “28% of wealth is valued manually and older than 90 days.”

Rules:
- no buy/sell suggestions
- no investment advice
- thresholds configurable if settings exist

Tests:
- single asset concentration
- currency concentration
- stale exposure
- no taxonomy behavior
- Islamic mode off/on behavior

Run validation and commit only when green.
```

---

# **Phase 5 — Web Evidence Engine**

## **Prompt 35 — Web Evidence foundation**

```
Read AGENTS.md and the full codebase first.

Goal:
Add safe web evidence support without paid APIs.

This is evidence collection for manually valued assets. It must not auto-update values.

Schema:
web_search_jobs:
- id
- asset_id
- status
- created_at
- completed_at nullable

web_search_queries:
- id
- job_id
- provider_id nullable
- raw_query
- executed_at nullable

web_search_results:
- id
- query_id
- url
- title nullable
- snippet nullable
- rank nullable

web_source_policies:
- id
- domain
- policy_type: allow | block | official
- created_at

web_source_reputation:
- id
- domain
- trust_score
- total_extractions
- approved_extractions
- rejected_extractions

web_fetched_pages:
- id
- url
- canonical_url nullable
- http_status nullable
- content_type nullable
- content_hash
- fetched_at

web_extracted_facts:
- id
- fetched_page_id
- fact_key
- raw_value
- confidence
- created_at

web_price_candidates:
- id
- extracted_fact_id
- normalized_price
- currency
- price_date nullable
- unit nullable

web_evidence_packs:
- id
- asset_id
- suggested_value nullable
- low_range nullable
- high_range nullable
- composite_score
- status: pending | approved | rejected | evidence_only
- created_at

web_evidence_reviews:
- id
- evidence_pack_id
- user_decision
- final_value nullable
- notes nullable
- reviewed_at

Rules:
- no paid API dependency
- no auto-mutating valuations
- no paywall/login bypass
- user approval required
- every candidate stores source URL/hash

Implement first:
- user-pasted URL evidence only
- static fetch mocked/tested
- evidence pack creation
- review states

Tests:
- job creation
- blocked domain rejected
- fetched page stored
- evidence cannot auto-update value
- approval writes valuation with provenance

Run validation and commit only when green.
```

---

## **Prompt 36 — SearXNG search provider**

```
Read AGENTS.md and the full codebase first.

Goal:
Add configurable SearXNG search without making it required.

Backend:
- search provider abstraction
- SearXNG provider:
  - endpoint URL from settings
  - JSON API
  - timeout
  - retry
  - rate limit
  - disabled by default unless configured
- no paid search default

Settings:
- web_evidence_enabled
- searxng_endpoint nullable
- allow_ai_query_expansion default false
- allow_background_web_jobs default false

Frontend:
- Settings > Web Evidence
- configure endpoint
- test connection
- disable web evidence

Rules:
- if no endpoint configured, only user-pasted URL works
- no Google/Bing paid API
- no fake search result

Tests:
- mocked SearXNG response
- timeout
- disabled provider
- invalid endpoint
- settings persist

Run validation and commit only when green.
```

---

## **Prompt 37 — Safe web fetcher and snapshot store**

```
Read AGENTS.md and the full codebase first.

Goal:
Fetch pages safely and store evidence snapshots.

Backend:
- reqwest static fetch first
- domain allow/block policy
- per-domain rate limit
- robots.txt best-effort check if feasible
- no login/paywall/captcha bypass
- content type detection
- canonical URL
- content hash
- cache policy
- delete cache function

Optional dynamic fetch:
- create interface for Playwright sidecar
- disabled by default
- do not implement stealth bypass
- do not break if not installed

Schema additions:
- web_page_snapshots
  - id
  - fetched_page_id
  - cleaned_text nullable
  - json_ld_blob nullable
  - metadata_json nullable
  - created_at

- web_rate_limits
  - id
  - domain
  - last_request_at
  - minimum_delay_ms

Tests:
- static fixture fetch
- blocked domain
- rate limit enforced
- failed fetch stored
- content hash deterministic
- cache delete

Run validation and commit only when green.
```

---

## **Prompt 38 — Web content extraction**

```
Read AGENTS.md and the full codebase first.

Goal:
Extract useful structured evidence from fetched pages.

Extraction priority:
1. JSON-LD
2. OpenGraph/meta tags
3. tables
4. main content/readability-style text
5. deterministic regex/heuristics
6. AI extraction only later

Implement:
- title extraction
- canonical URL extraction
- metadata extraction
- JSON-LD parsing
- OpenGraph parsing
- table extraction where feasible
- main text extraction via safe local library or deterministic fallback
- extraction version

Store:
- web_page_snapshots
- web_extracted_facts

Rules:
- extracted facts are candidates only
- no valuation write
- no fake confidence
- unsupported page = clear error/state

Tests with fixture HTML:
- property listing
- car listing
- watch listing
- fund factsheet page
- sukuk/bond page
- malformed HTML
- JSON-LD page

Run validation and commit only when green.
```

---

## **Prompt 39 — Web price candidate normalization**

```
Read AGENTS.md and the full codebase first.

Goal:
Normalize extracted public evidence deterministically.

Implement parsers:
- price
- currency symbol/ISO
- date
- unit
- property size/price per area
- car year/mileage/trim
- watch brand/reference/condition
- gold/silver weight/purity
- fund NAV/date
- bond/sukuk ISIN/maturity/coupon/profit rate

Implement:
- normalized candidate table inserts
- impossible value rejection
- stale date warning
- duplicate candidate detection
- MAD outlier rejection for multiple numeric candidates
- composite confidence scoring:
  - source trust
  - recency
  - identity match
  - currency match
  - unit match
  - source count
  - variance/outliers

Rules:
- property/car/watch produce range, not exact truth
- gold/silver show reference estimate only
- official sources rank highest
- user approval required

Tests:
- currency parsing
- unit conversion
- purity conversion
- outlier rejection
- stale page warning
- impossible values rejected

Run validation and commit only when green.
```

---

## **Prompt 40 — Web Evidence Review UI**

```
Read AGENTS.md and the full codebase first.

Goal:
Let users review public evidence before it affects asset values.

Frontend:
- asset detail button: “Find public evidence”
- “Paste source URL”
- Evidence Review page
- source cards:
  - domain
  - source class
  - title
  - extracted value
  - currency/date
  - snippet/citation
  - fetched_at
  - confidence
  - warnings
- range view:
  - low
  - median/suggested
  - high
  - outliers removed
- actions:
  - approve suggestion
  - edit and approve
  - reject
  - save evidence only
  - trust source
  - block source

Backend:
- approve_web_evidence
- reject_web_evidence
- save_evidence_only
- approval creates valuation with source_type web_evidence and citation/provenance
- audit event

Rules:
- no auto-update
- rejected evidence remains in history
- every approved valuation links back to evidence pack

Tests:
- render evidence pack
- approve writes valuation
- edit and approve writes edited value
- reject does not write valuation
- source block/trust works

Run validation and commit only when green.
```

---

## **Prompt 41 — Asset Web Watchlists**

```
Read AGENTS.md and the full codebase first.

Goal:
Let users track trusted public URLs for manually valued assets.

Schema:
asset_web_watchlists:
- id
- asset_id
- url
- domain
- check_frequency
- is_enabled
- last_checked_at nullable
- last_content_hash nullable
- last_evidence_pack_id nullable
- created_at
- updated_at

Backend:
- add_watch_url
- remove_watch_url
- enable/disable
- run_watch_check
- detect content hash/value changes
- generate alert/evidence pack, not valuation

Frontend:
- asset detail watchlist section
- add trusted URL
- check now
- last checked
- change detected

Rules:
- background checks off by default unless user enables
- no aggressive scraping
- no auto valuation update

Tests:
- create/update/delete
- check watched source
- changed content creates evidence
- unchanged content no spam
- disabled watch not checked

Run validation and commit only when green.
```

---

## **Prompt 42 — Property, car, watch comparable evidence**

```
Read AGENTS.md and the full codebase first.

Goal:
Add comparable range workflows for hard-to-price assets.

Property:
- approximate location by default
- exact address only with explicit user approval
- fields: location, property type, size, bedrooms, currency
- output range with asking-price warning

Car:
- make, model, year, trim, mileage, location, currency
- output median/range with condition caveat

Watch/collectible:
- brand, model, reference, year, condition, box/papers
- output range with liquidity/spread caveat

Backend:
- query template generation deterministic first
- AI expansion disabled unless future setting enabled
- normalize candidates
- outlier removal
- evidence pack creation

Frontend:
- guided “Find comparable evidence”
- privacy warning for location
- range and source cards
- approve/edit/reject

Tests:
- query templates
- privacy exact-address block
- range creation
- outlier removal
- approval flow

Run validation and commit only when green.
```

---

## **Prompt 43 — Gold and silver reference evidence**

```
Read AGENTS.md and the full codebase first.

Goal:
Support gold/silver valuation evidence without paid APIs.

Inputs:
- metal type
- weight
- unit: gram | ounce | kg
- purity: 24k | 22k | 18k | custom
- currency

Backend:
- configurable trusted public source URL
- user-pasted URL support
- extract spot/reference price
- normalize to grams/ounces/kg
- purity adjustment
- currency conversion only through existing FX service
- evidence pack and user approval

Rules:
- label as reference price
- physical premiums/spreads not guaranteed
- no live tick guarantee
- no auto-update

Tests:
- unit conversion
- purity conversion
- FX conversion missing path
- approved valuation provenance
- stale source warning

Run validation and commit only when green.
```

---

## **Prompt 44 — Sukuk, bond, fund factsheet evidence**

```
Read AGENTS.md and the full codebase first.

Goal:
Extract public factsheet evidence for fixed-income/funds.

Sources:
- user-pasted URL
- official issuer/fund/exchange pages
- fetched PDFs if allowed
- document vault link if downloaded/uploaded

Extract:
- ISIN
- issuer/fund name
- maturity date
- currency
- coupon/profit rate
- payment frequency
- NAV/date for funds
- expense ratio where available
- Shariah notes if explicitly present

Backend:
- factsheet evidence pack
- proposed metadata update requiring review
- optional document vault ingestion for fetched PDF where user approves
- no direct asset metadata mutation

Frontend:
- factsheet review screen
- approve metadata update
- link source document

Tests:
- fixture HTML factsheet
- fixture PDF if parser available
- missing ISIN warning
- stale factsheet warning
- approve metadata update

Run validation and commit only when green.
```

---

## **Prompt 45 — Web Evidence final hardening**

```
Read AGENTS.md and the full codebase first.

Goal:
Harden all web evidence flows.

Audit:
1. No paid API dependency.
2. No auto-mutating valuations.
3. No bypass of logins/paywalls/captcha.
4. Domain allow/block works.
5. Rate limits enforced.
6. Cache deletion works.
7. Privacy settings work.
8. Evidence packs always include source URL/fetched_at/hash.
9. User approval required for valuation write.
10. Rejected evidence retained.
11. Approved values carry provenance.
12. Tests cover failure states.

Run:
- pnpm typecheck
- pnpm lint
- pnpm test
- pnpm build
- cargo fmt
- cargo clippy -- -D warnings
- cargo test

Fix all issues and commit only when green.
```

---

# **Phase 6 — Local AI Intelligence Layer**

## **Prompt 46 — Intelligence settings and privacy center**

```
Read AGENTS.md and the full codebase first.

Goal:
Create full user control over local AI.

Extend Intelligence Settings:
- local_ai_enabled
- memory_enabled
- semantic_search_enabled
- local_model_inference_enabled
- offline_only_mode
- allow_background_ai_jobs
- allow_ai_to_summarize_reports
- allow_ai_to_suggest_memory
- allow_ai_query_expansion_for_web
- explanation_style: simple | standard | professional | accountant
- privacy_mode_enabled

Frontend:
- Intelligence Settings page
- Privacy Center page
- clear toggles
- “Local-only” badge
- delete AI cache
- export AI data
- reset memory
- disable all AI

Backend:
- settings commands
- clear_ai_cache command placeholder only if real tables exist; otherwise returns clean unsupported
- export_ai_data where applicable

Rules:
- AI off means no jobs run
- offline-only mode blocks network triggered by AI
- web evidence has separate explicit toggle

Tests:
- settings defaults
- toggle persistence
- AI disabled prevents job creation
- offline-only blocks web query planning

Run validation and commit only when green.
```

---

## **Prompt 47 — AI guardrails and suggestion records**

```
Read AGENTS.md and the full codebase first.

Goal:
Create the safety firewall before any AI runtime exists.

Schema:
ai_suggestion_records:
- id
- suggestion_type
- target_domain
- target_entity_type nullable
- target_entity_id nullable
- source_object_ids_json
- model_id nullable
- prompt_version nullable
- suggested_data_json
- confidence nullable
- status: pending | approved | rejected | expired
- guardrail_status: pass | flagged | rejected
- output_hash
- created_at
- reviewed_at nullable

ai_guardrail_violations:
- id
- suggestion_id nullable
- violation_type
- details_json
- created_at

prompt_templates:
- id
- name
- version
- purpose
- template_text
- output_schema_json
- created_at

Backend:
- validate_ai_output_against_schema
- create_ai_suggestion
- reject_guardrail_violation
- approve_suggestion only into allowed suggestion workflows, not direct ledger
- deny writes to protected domains

Protected domains:
- activities
- holdings
- valuations
- tax_pack_lines
- shariah verdicts
- approved extracted_facts
- source documents
- report finalized lines

Tests:
- AI cannot directly write activities
- AI cannot approve facts
- invalid JSON rejected
- uncited numeric claim rejected where citations required
- guardrail violation logged

Run validation and commit only when green.
```

---

## **Prompt 48 — Local AI model registry finalization**

```
Read AGENTS.md and the full codebase first.

Goal:
Complete local model registry for ONNX/GGUF/optional external runtimes.

Enhance existing registry:
- runtime availability detection
- hardware/RAM check
- model capability matrix
- checksum verification before enable
- signed manifest support if practical
- disabled state if runtime missing

Supported model categories:
- embedding
- classifier
- summarizer
- JSON extractor
- simplifier
- assistant
- reranker

Frontend:
- install/register model from local file
- show runtime type
- show file size
- show checksum verified/unverified
- show capabilities
- show “not loaded” / “runtime unavailable”

Rules:
- no model download yet unless existing safe downloader exists
- no paid models
- no cloud inference
- no fake model availability

Tests:
- RAM check path
- runtime unavailable path
- checksum mismatch
- capability filtering

Run validation and commit only when green.
```

---

## **Prompt 49 — AI job system**

```
Read AGENTS.md and the full codebase first.

Goal:
Implement robust background AI jobs.

Schema:
ai_jobs:
- id
- job_type: embed_object | classify_document | generate_briefing | generate_memory_candidate | summarize_alert | draft_report_commentary | web_query_plan | review_copilot
- status: queued | running | succeeded | failed | cancelled
- priority
- input_payload_json
- source_object_type nullable
- source_object_id nullable
- model_id nullable
- attempts
- max_attempts
- error_message nullable
- created_at
- started_at nullable
- completed_at nullable

ai_job_attempts:
- id
- job_id
- attempt_number
- status
- started_at
- completed_at nullable
- error_message nullable

ai_job_outputs:
- id
- job_id
- output_json
- output_hash
- suggestion_id nullable
- created_at

Backend:
- enqueue_ai_job
- run_next_ai_job
- cancel_ai_job
- list_ai_jobs
- retry_failed_job
- do not run if local_ai_enabled false
- model capability check before running
- no direct financial mutation

Frontend:
- Intelligence Jobs status under settings
- retry/cancel
- error display

Tests:
- disabled AI blocks job
- queued/running/succeeded lifecycle
- failed job retry
- unsupported capability rejected
- output stored with hash

Run validation and commit only when green.
```

---

## **Prompt 50 — Semantic index with FTS5 and vector-ready abstraction**

```
Read AGENTS.md and the full codebase first.

Goal:
Create local exact + semantic search infrastructure.

Schema:
semantic_index_items:
- id
- object_type
- object_id
- title
- body_text
- content_hash
- visibility_scope_json nullable
- created_at
- updated_at

semantic_links:
- id
- source_item_id
- target_object_type
- target_object_id
- link_type
- created_at

semantic_embeddings:
- if sqlite-vec available, create vector table
- if unavailable, create capability detection and no-op fallback
- store embedding model id, vector dimensions, content hash

Backend:
- index_object
- remove_object_from_index
- reindex_all
- exact_search using SQLite FTS5 if available
- vector_search if sqlite-vec available
- hybrid_search merging exact/vector results
- permission/entity filters before returning results

Index:
- assets
- accounts
- activities
- alerts
- reports
- documents/text blocks
- extracted facts
- web evidence packs
- memory items if approved

Rules:
- app works without sqlite-vec
- exact search must work first
- no fake embeddings
- no cloud embeddings

Tests:
- index asset
- search exact text
- delete removes item
- reindex updates hash
- vector unavailable fallback
- visibility filter

Run validation and commit only when green.
```

---

## **Prompt 51 — Local memory layer**

```
Read AGENTS.md and the full codebase first.

Goal:
Implement safe, inspectable local personalization memory.

Schema:
ai_memory_items:
- id
- memory_type: preference | pattern | style | workflow | dismissed_alert_pattern | dashboard_preference | document_pattern
- encrypted_payload
- summary
- source_event_ids_json
- approved_at
- created_at
- updated_at
- deleted_at nullable

ai_memory_candidates:
- id
- memory_type
- candidate_payload_json
- summary
- why_suggested
- source_event_ids_json
- status: pending | approved | rejected
- created_at
- reviewed_at nullable

ai_memory_feedback:
- id
- memory_id nullable
- candidate_id nullable
- feedback_type
- created_at

ai_behavior_events:
- id
- event_type
- entity_type nullable
- entity_id nullable
- metadata_json
- created_at

ai_user_preferences:
- id
- preference_key
- preference_value_json
- source: explicit | inferred_approved
- created_at
- updated_at

Backend:
- record_behavior_event
- create_memory_candidate
- approve_memory_candidate
- reject_memory_candidate
- list_memory_items
- delete_memory_item
- export_memory
- reset_memory

Frontend:
- Memory Center
- “What Mizan remembers”
- “Why remembered?”
- approve/reject pending memory
- delete/export/reset

Rules:
- memory_enabled required
- sensitive memory requires explicit approval
- memory cannot create financial facts
- memory cannot mutate ledger
- local/encrypted where feasible

Tests:
- candidate lifecycle
- approve/reject
- delete/export/reset
- disabled memory blocks candidate generation
- memory not exposed in advisor/accountant scope

Run validation and commit only when green.
```

---

## **Prompt 52 — Ask Mizan Privately**

```
Read AGENTS.md and the full codebase first.

Goal:
Create private local search over Mizan data.

Frontend:
- global “Ask Mizan privately…” command/search
- local-only badge
- no-advice disclaimer
- result list with citations/source links
- filters:
  - Documents
  - Assets
  - Activities
  - Reports
  - Alerts
  - Web Evidence
  - Memory

Backend:
- private_search(query, filters)
- use exact FTS first
- use vector/hybrid if available
- return cited results only
- no generative answer yet unless local model available and guarded
- if AI disabled, search still works exactly

Rules:
- do not answer financial facts without sources
- no cloud search
- no fake citations
- role/entity filters respected

Tests:
- search assets
- search document text
- source links returned
- empty query handled
- permission filter applied
- AI disabled still returns exact results

Run validation and commit only when green.
```

---

## **Prompt 53 — Deterministic Daily Wealth Briefing**

```
Read AGENTS.md and the full codebase first.

Goal:
Build deterministic daily briefing foundation.

Briefing sections:
- greeting
- what changed
- needs attention
- income received
- upcoming events
- document reviews
- stale valuations
- tax readiness
- zakat readiness if enabled
- one next best action

Schema:
briefings:
- id
- briefing_date
- status
- created_at

briefing_sections:
- id
- briefing_id
- section_type
- title
- body
- source_refs_json
- created_at

Backend:
- generate_daily_briefing
- all numbers from deterministic services
- source_refs for every number
- no AI wording yet unless hook exists but disabled

Frontend:
- Home briefing card
- refresh button
- click sources
- empty state

Rules:
- no investment advice
- no fake sections
- if data missing, say what is missing

Tests:
- empty portfolio
- stale valuations
- active alerts
- private capital call if implemented
- Islamic mode off/on
- no uncited number

Run validation and commit only when green.
```

---

## **Prompt 54 — Next Best Action engine**

```
Read AGENTS.md and the full codebase first.

Goal:
Tell users the single most important next action.

Schema:
next_best_actions:
- id
- action_type
- title
- explanation
- priority_score
- source_entity_type
- source_entity_id
- status: open | done | snoozed | dismissed
- action_route
- created_at
- updated_at
- snoozed_until nullable

Ranking inputs:
- alert severity
- due dates
- stale valuation age
- pending document reviews
- tax pack missing data
- private capital calls
- upcoming maturities
- data quality deductions
- approved memory preferences if enabled

Backend:
- calculate_next_best_actions
- deterministic scoring
- no AI required
- action routes validated

Frontend:
- Home next-action card
- secondary actions list
- snooze/dismiss/done

Tests:
- critical due item ranks first
- stale valuation lower priority than due capital call
- dismissed hidden
- snooze works
- action route valid

Run validation and commit only when green.
```

---

## **Prompt 55 — Senior-mode AI wording hook**

```
Read AGENTS.md and the full codebase first.

Goal:
Add AI/plain-language wording safely, after deterministic facts exist.

Settings:
- explanation_style already exists:
  - simple
  - standard
  - professional
  - accountant

Implement:
- deterministic explanation object first
- optional AI wording hook creates suggestion only
- if no local model, use deterministic template fallback
- AI output must preserve numbers/citations exactly
- validate output contains no new numeric claims unless present in source payload

Use cases:
- alert explanation
- briefing wording
- Explain This Number plain-English section
- senior mode simplification

Rules:
- AI cannot change calculations
- AI cannot add new numbers
- AI cannot remove warnings
- invalid output rejected
- no cloud AI

Tests:
- simple/professional mode renders
- numbers preserved exactly
- citation refs preserved
- hallucinated number rejected
- fallback template works without model

Run validation and commit only when green.
```

---

## **Prompt 56 — AI Document Triage**

```
Read AGENTS.md and the full codebase first.

Goal:
Classify uploaded documents and route them intelligently.

Inputs:
- document metadata
- filename
- mime type
- extracted text if available
- text blocks/tables if available

Outputs:
- document_type suggestion:
  - broker_statement
  - bank_statement
  - tax_form
  - dividend_voucher
  - sukuk_factsheet
  - bond_termsheet
  - fixed_deposit_receipt
  - capital_call_notice
  - distribution_notice
  - private_fund_nav
  - insurance_statement
  - ulip_statement
  - property_valuation
  - loan_statement
  - unknown
- likely account/asset links
- date range
- tax year
- extraction priority
- confidence
- review_required

Backend:
- deterministic heuristics first
- local classifier optional second
- store output as ai_suggestion_record
- no truth mutation

Frontend:
- document detail shows triage suggestion
- user approve/edit/reject
- approved mapping may create document_link only

Tests:
- fixture filenames/text for each type
- low confidence path
- approve link
- reject suggestion
- disabled AI uses heuristics only

Run validation and commit only when green.
```

---

## **Prompt 57 — AI Review Copilot**

```
Read AGENTS.md and the full codebase first.

Goal:
Assist document review without approving anything automatically.

In Review Queue, show:
- suggested mapping
- confidence
- duplicate/mismatch warnings
- source citation
- plain-English explanation
- comparison to existing ledger/valuation
- recommended action wording:
  - approve
  - edit
  - reject
  - needs more info

Backend:
- review_copilot_suggestion
- uses deterministic reconciliation first
- AI can summarize only
- suggestion record created
- user must approve/edit/reject

Memory:
- if user repeatedly approves same mapping, create memory candidate
- user approval required for memory

Rules:
- copilot cannot approve fact
- copilot cannot write ledger
- invalid uncited output rejected

Tests:
- duplicate warning
- mismatch warning
- approved path still requires user action
- memory candidate only after permission
- disabled AI fallback works

Run validation and commit only when green.
```

---

## **Prompt 58 — AI report commentary**

```
Read AGENTS.md and the full codebase first.

Goal:
Draft report commentary from deterministic report lines.

Reports:
- Monthly Wealth Letter
- Net Worth Report
- Tax Pack Summary
- Zakat Report
- Private Investment Report
- Data Quality Report

Backend:
- create_report_commentary_draft(report_run_id)
- input is deterministic report sections/lines only
- AI may draft text
- every numeric claim must map to report line
- commentary status: draft | accepted | rejected | edited

Frontend:
- report preview shows draft commentary separately
- user can edit/accept/reject
- audit event

Rules:
- no uncited numbers
- no advice
- no new facts
- no tax/legal recommendations
- no cloud AI

Tests:
- commentary draft uses provided numbers
- hallucinated number rejected
- reject/edit/accept lifecycle
- no model fallback uses deterministic template

Run validation and commit only when green.
```

---

## **Prompt 59 — AI \+ Web Evidence guardrails**

```
Read AGENTS.md and the full codebase first.

Goal:
Safely integrate local AI with Web Evidence.

AI may:
- generate query plans
- expand query variants
- classify page relevance
- summarize sources
- explain evidence pack
- draft warning text

AI cannot:
- fetch forbidden pages
- bypass policies
- invent prices
- update valuations
- update ledger
- approve evidence
- ignore missing citations

Structured schemas:
- WebQueryPlan
- WebSourceCandidate
- WebExtractedPrice
- WebEvidenceWarning
- ValuationSuggestion

Backend:
- all AI web outputs must reference fetched_page_id or search_result_id
- reject uncited values
- reject impossible values
- reject source not fetched
- log guardrail violations

Frontend:
- evidence review shows AI summary as secondary, never primary
- clear “AI summary, verify sources” label

Tests:
- AI query plan with PII rejected unless permission
- uncited price rejected
- impossible price rejected
- direct valuation mutation impossible
- guardrail logs written

Run validation and commit only when green.
```

---

## **Prompt 60 — Memory Safety Center**

```
Read AGENTS.md and the full codebase first.

Goal:
Make memory transparent and trustworthy.

Frontend:
Memory Safety Center:
- what Mizan remembers
- pending memory suggestions
- why remembered
- where used
- delete memory
- export memory
- reset personalization
- disable memory
- privacy mode

Backend:
- list_memory_usage(memory_id)
- delete_memory
- export_memory
- reset_all_memory
- disable_memory_and_stop_candidates

Rules:
- disabling memory stops future memory candidates
- delete means memory no longer influences dashboard/alerts
- export readable JSON
- sensitive memory requires approval

Tests:
- memory displayed
- why remembered displayed
- delete removes influence
- reset clears all
- disabled mode blocks use

Run validation and commit only when green.
```

---

# **Phase 7 — Entitlements and final hardening**

## **Prompt 61 — Entitlement abstraction only, no billing**

```
Read AGENTS.md and the full codebase first.

Goal:
Prepare feature gates without Stripe, registration, checkout, or billing.

Do not add:
- Stripe
- checkout
- registration
- subscription backend
- cloud account system

Add:
- FeatureEntitlement enum
- EntitlementProvider service
- LocalEntitlementSnapshot
- EntitlementGate frontend component/hook
- feature usage limits

Feature gates:
- Document Vault advanced extraction
- Tax Packs
- Private Investments
- Fixed Income/Sukuk advanced engine
- Shariah/Zakat
- Report Builder
- Estate Binder
- Web Evidence background checks
- Semantic Search
- Memory/AI features
- AI Report Commentary

Default:
- dev mode can enable all via local config
- free mode shows graceful gated UI
- no data hidden/lost if premium disabled; only premium actions gated

Tests:
- free user blocked from premium command
- dev override works
- gated UI renders
- existing free functionality works
- no billing code introduced

Run validation and commit only when green.
```

---

## **Prompt 62 — Offline signed license validation**

```
Read AGENTS.md and the full codebase first.

Goal:
Implement local license verification without billing rails.

Add:
license_entitlements:
- id
- license_key_hash
- tier
- features_json
- expires_at nullable
- signature
- verified_at
- created_at

Backend:
- use ed25519-dalek or existing crypto equivalent
- import_license_string
- verify signature using embedded public key
- test/dev key separation
- store local entitlement snapshot
- handle expired license
- handle invalid signature

Rules:
- no Stripe
- no registration
- no online activation
- no hostile DRM
- app data remains accessible if license expires
- only premium actions gated
- tests use generated test keys, not empty placeholders

Frontend:
- Settings > License
- paste license
- verify
- show tier/features/expiry
- invalid/expired messages

Tests:
- valid signed license accepted
- invalid signature rejected
- expired license rejected
- free tier usable
- premium gates unlock with valid license

Run validation and commit only when green.
```

---

## **Prompt 63 — Full final hardening pass**

```
Read AGENTS.md and the full codebase first.

Goal:
Stabilize mizan-smart into a serious working app.

Audit all new systems:
1. Core UI still works.
2. Existing imports still work.
3. Portfolio math still passes.
4. Assets still create/list/update.
5. Document Vault stores encrypted files.
6. Extracted facts require review.
7. Citations never fake source links.
8. Explain This Number never invents lineage.
9. Reconciliation never auto-writes.
10. Private investment metrics use Decimal.
11. Fixed income schedules use Decimal.
12. Islamic mode is optional and hidden when disabled.
13. Tax packs are data-prep only, not advice.
14. Web Evidence never auto-updates valuations.
15. AI cannot mutate protected financial tables.
16. AI outputs are structured, validated, auditable.
17. Memory is inspectable/deletable/exportable.
18. Semantic search respects scope/visibility.
19. Entitlement gates do not delete or hide user-owned data.
20. No paid API dependency added.
21. No cloud AI dependency added.
22. No `any`.
23. No f32/f64 money paths.
24. No fake rows/placeholders.

Run:
- pnpm typecheck
- pnpm lint
- pnpm test
- pnpm build
- cargo fmt
- cargo clippy -- -D warnings
- cargo test

Fix all failures properly.
Remove dead code.
Remove unused dependencies.
Update docs:
- docs/mizan-smart-architecture.md
- docs/local-ai-guardrails.md
- docs/web-evidence-policy.md
- docs/validation.md

Commit final stable milestone only after all checks pass.
```

---

# **Exact build order**

Use this order:

```
1. Stabilize mizan-smart baseline
2. Boomer-friendly navigation
3. Home dashboard
4. Universal asset model
5. Universal Add Asset
6. Manual valuations/bulk update
7. Data Quality Score
8. Smart alerts
9. Wealth Inbox
10. Document Vault storage
11. Document job system
12. PDF/text/layout extraction adapter
13. Extracted facts/citations
14. Document Review Queue
15. Explain This Number
16. Reconciliation Center
17. Private investments foundation
18. Private investment UI/J-curve
19. Fixed income/Sukuk/FD engine
20. Liquidity Ladder
21. Corporate actions
22. Accuracy invariants
23. Golden import templates
24. Islamic mode
25. Shariah screening
26. Zakat
27. Purification
28. Tax packs
29. CPA exports
30. Report Builder
31. Monthly Wealth Letter
32. Estate Binder
33. Fee Intelligence
34. Concentration Radar
35. Web Evidence foundation
36. SearXNG provider
37. Safe fetch/snapshots
38. Web extraction
39. Price normalization
40. Evidence review UI
41. Asset web watchlists
42. Property/car/watch evidence
43. Gold/silver evidence
44. Sukuk/bond/fund factsheet evidence
45. Web Evidence hardening
46. Intelligence settings/privacy
47. AI guardrails
48. Local model registry
49. AI job system
50. Semantic index
51. Local memory
52. Ask Mizan Privately
53. Daily briefing
54. Next Best Action
55. Senior-mode wording hook
56. AI document triage
57. AI Review Copilot
58. AI report commentary
59. AI + Web Evidence guardrails
60. Memory Safety Center
61. Entitlement abstraction
62. Offline license validation
63. Final hardening
```

# **What not to build yet**

```
Do not build yet:
- Stripe
- checkout
- registration
- cloud account management
- hosted AI
- paid search APIs
- aggressive broker sync expansion
- autonomous AI ledger updates
- investment advice assistant
- tax filing submission
- Shariah certification claims
- mobile polish before desktop product works
```

This is the clean path: **build the full wealth OS, then add AI on top as a safe intelligence layer, not as the financial brain.**

