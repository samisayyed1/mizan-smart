# Rolling back mizan-smart

This branch is experimental. Every commit can be undone — there are
two anchor points preserved as git tags so you never lose the way
back to a stable Mizan.

## The two anchors

| Tag | Commit | What it is | When to use it |
|---|---|---|---|
| `v3.4.1` | `e78c0b6` | The **public Mizan 3.4.1 release** | Roll back the *entire* product — including the upstream `mizan-4` tier-1 / tier-2 hardening PRs (#23, #24, #25) — to the last published release. |
| `pre-mizan-smart` | `4c3cdd6` | The **mizan-4 stable head** we forked from | Roll back **only the mizan-smart work**, keep the upstream tier-1 / tier-2 hardening that landed on mizan-4 after the public 3.4.1 release. **Recommended default.** |

Both tags are pushed to `origin` so they're recoverable even if your
local clone is gone.

## Quick rollback

```bash
# Recommended: undo only the mizan-smart work, keep upstream hardening
bash scripts/rollback.sh

# Or be explicit:
bash scripts/rollback.sh --to=pre-mizan-smart

# Full revert all the way back to the public 3.4.1 release:
bash scripts/rollback.sh --to=v3.4.1

# Preview without doing anything:
bash scripts/rollback.sh --dry-run
```

The script:
1. Refuses to run if you have uncommitted changes.
2. Saves your **current branch state** as `rollback-backup-<UTC timestamp>` so nothing is lost.
3. Hard-resets the current branch to the target tag.
4. Reinstalls pnpm deps to match the rolled-back lockfile.

After rolling back, your previous work is reachable via:
```bash
git checkout rollback-backup-<timestamp>
```

## Manual rollback (without the script)

```bash
# Save your work
git branch my-backup HEAD
# Reset
git reset --hard pre-mizan-smart           # or v3.4.1
# Restore deps
pnpm install --frozen-lockfile
```

## Restoring after a rollback

```bash
# List any backup branches the script created
git branch --list 'rollback-backup-*'

# Restore the most recent one
git reset --hard rollback-backup-<timestamp>
pnpm install --frozen-lockfile
```

## Pushing the rollback to origin

If you want the remote to reflect the rolled-back state:

```bash
git push origin main --force-with-lease
```

Use `--force-with-lease` (not bare `--force`) so the push is rejected
if anyone else has pushed in the meantime.

## What gets undone

Rolling back to `pre-mizan-smart` removes (in chronological order):

1. `chore(phase-0)` — baseline stabilization (clippy fixes, plan docs)
2. `feat(phase-1/p2)` — senior-friendly primary navigation
3. `feat(phase-1/p3)` — Home dashboard Quick Actions + Inbox preview
4. `feat(phase-1/p8)` — Smart Alerts foundation
5. `feat(phase-1/p7)` — Data Quality Score
6. `feat(phase-1/p4)` — Universal asset model
7. `chore(disk)` — disk hygiene + scripts
8. *(plus any later phase commits)*

Rolling back to `v3.4.1` additionally removes the upstream mizan-4
PRs #23, #24, #25 (production hardening, tier-1 hardening, tier-2
hardening).
