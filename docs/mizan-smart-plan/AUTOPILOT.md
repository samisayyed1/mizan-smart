# Mizan Autopilot Runbook v3

This file is the durable source of truth. The live Codex prompt only
bootstraps it. Every iteration re-reads this file from disk first.

## Identity

CWD: /Users/samisayyed/Documents/mizan-smart
Origin: https://github.com/samisayyed1/mizan-smart
Branch: main (never check out anything else)
Plan: docs/mizan-smart-plan/PLAN.md (63 prompts, 8 phases)
Conventions: AGENTS.md
Progress: docs/mizan-smart-plan/PROGRESS.md (source of truth for
"what's next")
State: docs/mizan-smart-plan/.autopilot-state.json
Rollback anchors (DO NOT TOUCH): tags `v3.4.1` and `pre-mizan-smart`

## Stop conditions (ONLY these three stop the loop)

STOP-DONE:
  grep -c '^- \[ \]' docs/mizan-smart-plan/PROGRESS.md = 0
  AND docs/mizan-smart-plan/FINAL-MANIFEST.md exists on HEAD
  AND `git rev-parse HEAD` == `git rev-parse origin/main`

STOP-BLOCKED:
  state.json shows `attempts_on_current` >= 10
  AND the last 3 attempts failed the same validation command with the
  same error hash. Write docs/mizan-smart-plan/BLOCKER.md, push, stop.

STOP-PAUSE:
  File docs/mizan-smart-plan/PAUSE.md exists. This is a human-driven
  pause for audit / polish. Do NOT start a new iteration. Do NOT
  attempt to remove the file. Simply exit. The human will remove
  the file when work should resume. Until then, every iteration is
  a no-op.

Nothing else stops the loop. Not "the task feels done". Not "the
context is getting big". Not "a single test failed". Not "the network
flaked". Not "the disk is full".

## Iteration template

For every iteration, run these steps in order. Each step is
idempotent — re-running it on a partially-completed iteration is
safe.

### STEP 0 — Re-read durable state
  - `cd /Users/samisayyed/Documents/mizan-smart`
  - **PAUSE GATE:** If `docs/mizan-smart-plan/PAUSE.md` exists, exit
    immediately. Print one line: `AUTOPILOT-PAUSED`. Do nothing else.
    The human controls resumption by deleting that file.
  - `git rev-parse --abbrev-ref HEAD` must equal `main`. If not,
    `git checkout main`.
  - `git rev-parse --verify v3.4.1^{commit}` must succeed.
  - `git rev-parse --verify pre-mizan-smart^{commit}` must succeed.
  - Read AUTOPILOT.md, PROGRESS.md, .autopilot-state.json.
  - Read AGENTS.md.

### STEP 1 — Sync origin
  - `git fetch origin`
  - If origin/main is ahead, `git pull --ff-only origin main`.
  - If a previous iteration has unpushed commits, push them now using
    STEP 7's retry policy before starting a new prompt.

### STEP 2 — Recover from any partial work
  - If `git status --porcelain` is non-empty:
      Decide:
        (a) If the changes belong to the prompt named in
            state.json.current_prompt AND validation passes, continue
            from STEP 5 (commit). This handles the case where
            Codex died after edits but before commit.
        (b) Otherwise, stash to `autopilot-recovery-$(date -u +%s)`.
            Do not drop the stash — it is recoverable evidence.

### STEP 3 — Pick the next prompt
  - `next=$(grep -n '^- \[ \]' docs/mizan-smart-plan/PROGRESS.md | head -1)`
  - If `next` is empty, jump to FINALISATION.
  - Parse it into prompt_num (e.g. p15), commit_tag (e.g.
    phase-2/p15), and title.
  - Update state.json: set current_prompt=p15, attempts_on_current
    incremented by 1 (or reset to 1 if previous was a different
    prompt), reset last_error_hash to null on fresh prompt.
  - Commit nothing yet — state.json is allowed to evolve mid-iteration.

### STEP 4 — Read what you need to implement
  - Open PLAN.md and locate the "## **Prompt N — Title**" section.
    Read it fully.
  - Locate the most-recently-committed similar feature as a template.
    Good templates:
      - crates/core/src/universal_assets      typed domain + traits
      - crates/storage-sqlite/src/universal_assets  typed repo + tests
      - crates/core/src/alerts                engine + rule trait
      - crates/core/src/data_quality          pure deterministic service
      - apps/tauri/src/commands/universal_asset.rs  Tauri shape
      - apps/server/src/api/universal_assets.rs     Axum shape
      - apps/frontend/src/adapters/shared/universal-assets.ts adapter
      - apps/frontend/src/pages/holdings/new   page + zod + RHF + tests
  - Read the existing files in those modules end-to-end before editing.

### STEP 5 — Implement the FULL vertical slice
  Touch only what the prompt requires, but every layer that needs
  touching:
    - SQL migration (up.sql + down.sql, FK constraints, indexes,
      CHECK constraints)
    - Diesel schema (table!, joinable!,
      allow_tables_to_appear_in_same_query!)
    - crates/core domain types + traits + tests
    - crates/storage-sqlite repository + tests
      (WriteHandle::exec_tx for multi-table mutations)
    - apps/tauri command + module registration + invoke_handler!
    - apps/server endpoint + module registration + router merge
    - apps/frontend adapter (shared, re-exported from both tauri and
      web), web/core.ts dispatch case if needed
    - apps/frontend page/component using react-hook-form + zod when
      forms are involved, route in routes.tsx if new
    - tests on BOTH sides

  If the prompt depends on an external runtime that is not present
  (local LLM, OCR sidecar, SearXNG, etc.):
    - Build the trait + detection function + disabled-state UI +
      tests covering both branches. Do NOT fake success. Commit that.

### STEP 6 — Validation gate (every command exit-code 0)
  Run IN ORDER. Hash any failing command's last 40 lines of stderr
  into last_error_hash before retrying.

    1.  cargo fmt --all
    2.  cargo clippy --workspace --all-targets -- -D warnings
    3.  cargo test --workspace
    4.  pnpm type-check
    5.  pnpm lint
    6.  CI=true pnpm --filter frontend exec vitest run
    7.  pnpm build

  Retry classification:
    TRANSIENT (retry up to 5 times, no attempt counter increment):
      - test timeout under laptop load
      - SQLite "database is locked"
      - DNS / TLS / git-fetch flake
      - cargo "failed to acquire packages cache lock"
      - pnpm ENOENT temp file
      - OOM kill
    REAL (counts toward STOP-BLOCKED):
      - compile errors
      - clippy warnings (anything --D warnings catches)
      - test failures with stable diffs twice
      - type-check errors
      - lint errors

  When fixing REAL failures, fix the ROOT CAUSE. NEVER add
  #[allow(...)], `eslint-disable`, `@ts-ignore`, `it.skip`, `vi.skip`,
  `#[ignore]`, or `CI=false` to silence them. The single permitted
  exception: `#[allow(dead_code)]` on a struct field documented as
  "Reserved for prompt pN" — and only that.

### STEP 7 — Tick PROGRESS.md atomically, commit, push

  7a. Tick: replace `- [ ] pN ...` with `- [x] pN ... (PENDING-SHA)`
  7b. `git add -A`
  7c. Commit with subject:
        feat(<commit_tag>): <one-line summary>
      (chore(phase-0) for prompt 1 only)
      Multi-paragraph body explaining migration, types, repo, command,
      endpoint, frontend, tests, validation outcome.
  7d. `sha=$(git rev-parse --short HEAD)`
  7e. Replace `PENDING-SHA` in PROGRESS.md with `$sha`, stage, and
      `git commit --amend --no-edit` (allowed because the commit is
      still local).
  7f. Push with retry:
        delay=2
        for attempt in 1..10:
          if `git push origin main`:
            verify HEAD == origin/main
            break
          else:
            sleep $delay; delay=$((delay*2))
            git fetch origin
            if origin moved: git pull --rebase origin main, re-run
            validation (STEP 6). If that fails, REAL failure.
        If 10 attempts fail, write BLOCKER.md (network diagnostics),
        STOP.

### STEP 8 — Update state, disk guard, loop
  - state.json: current_prompt=null, attempts_on_current=0,
    last_push_sha=$(git rev-parse --short origin/main)
  - `size_gb=$(du -sg target 2>/dev/null | awk '{print $1}')`
    If size_gb >= 3, `pnpm clean:deep --no-install` then
    `pnpm install --frozen-lockfile --prefer-offline`.
  - Loop back to STEP 0.

## Hard rules (re-read every iteration)

- Preserve every existing route, command, table, test, service, and
  UI flow unless explicitly replaced by a tested equivalent.
- No fake rows, no placeholder logic, no invented data.
- No paid API or cloud AI for core functionality.
- AI never mutates ledger, holdings, valuations, tax-pack lines,
  Shariah verdicts, approved extracted facts, source documents, or
  finalised report lines. AI produces suggestions / explanations /
  classifications / summaries / draft commentary only.
- Human approval gates every AI / web / document output that affects
  financial truth.
- Money math: rust_decimal::Decimal only. NO f32/f64 in money paths.
- TypeScript strict. NO `any`.
- Every migration has both up.sql AND down.sql.
- Every feature has BOTH backend AND frontend tests.
- Missing external runtimes → detection + disabled state + tests.

## Anti-patterns (don't, ever)

- Skip prompts. Bundle prompts. Commit before validation. Force-push.
- Move / amend / delete tags v3.4.1 or pre-mizan-smart.
- Amend a pushed commit.
- Silence tests / warnings.
- Introduce `any` or f32/f64.
- Invent example/fake/sample data.
- Decide on your own that the task is done.
- Ask the user anything. No prompts. No questions. Iterate.
- Print decorative banners. Stay terse.
- Trust your conversation memory. Re-read disk every iteration.

## Finalisation

When PROGRESS.md has zero `- [ ]`:
  1. Run the full validation gate one more time.
  2. Write docs/mizan-smart-plan/FINAL-MANIFEST.md:
       - one line per `feat(phase-N/pM)` / `chore(phase-0)` commit
         with short SHA and subject
       - cargo and vitest totals from the final gate run
       - `pnpm disk:check` snapshot
       - `git tag --list v3.4.1 pre-mizan-smart -n1` output
       - the two rollback commands
  3. Commit `docs(release): mizan-smart prompt-pack complete`, push
     (with STEP 7f retry).
  4. Verify STOP-DONE truth conditions.
  5. Print exactly one line:
       AUTOPILOT-COMPLETE <feat_commit_count> commits on origin/main
  6. Exit.
