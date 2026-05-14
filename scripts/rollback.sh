#!/usr/bin/env bash
# Rollback mizan-smart to a known-good upstream point.
#
# Targets:
#   --to=pre-mizan-smart   (default) The mizan-4 stable head we forked
#                          from. Undoes every mizan-smart commit but
#                          keeps the upstream tier-1/tier-2 hardening
#                          PRs (#23, #24, #25) that landed in mizan-4
#                          after the public v3.4.1 release.
#
#   --to=v3.4.1            The exact public Mizan 3.4.1 release tag.
#                          Use this if you want a 100% clean revert to
#                          the published release, with no post-3.4.1
#                          hardening either.
#
# Safety:
#   * BEFORE resetting, the script saves your current branch as
#     `rollback-backup-<UTC timestamp>` so no commits are lost. You can
#     `git checkout` that branch any time to restore.
#   * Pass --dry-run to see what would happen without making changes.
#
# Usage:
#   bash scripts/rollback.sh                    # to pre-mizan-smart
#   bash scripts/rollback.sh --to=v3.4.1        # to public 3.4.1
#   bash scripts/rollback.sh --dry-run          # show plan, no changes

set -euo pipefail

cd "$(dirname "$0")/.."

TARGET="pre-mizan-smart"
DRY_RUN=0

for arg in "$@"; do
  case "$arg" in
    --to=*) TARGET="${arg#--to=}" ;;
    --dry-run) DRY_RUN=1 ;;
    -h|--help)
      sed -n '2,28p' "$0"
      exit 0
      ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

# Validate target exists.
if ! git rev-parse --verify "$TARGET" >/dev/null 2>&1; then
  echo "error: target '$TARGET' does not exist as a ref" >&2
  echo "       Available rollback points:" >&2
  git tag --list "v3.*" "pre-mizan-smart" | sed 's/^/         /' >&2
  exit 1
fi

# Dereference annotated tags to the actual commit they point at so
# the "Already at target" check is honest.
TARGET_SHA="$(git rev-parse "${TARGET}^{commit}")"
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
CURRENT_SHA="$(git rev-parse HEAD)"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP_BRANCH="rollback-backup-${TIMESTAMP}"

echo "Rollback plan"
echo "-------------"
echo "  current branch:  ${CURRENT_BRANCH}"
echo "  current HEAD:    ${CURRENT_SHA}"
echo "  rolling back to: ${TARGET} (${TARGET_SHA})"
echo "  backup branch:   ${BACKUP_BRANCH}"
echo ""

if [[ "$CURRENT_SHA" == "$TARGET_SHA" ]]; then
  echo "Already at target; nothing to do."
  exit 0
fi

if [[ $DRY_RUN -eq 1 ]]; then
  echo "[dry-run] would:"
  echo "  1. git branch ${BACKUP_BRANCH} ${CURRENT_SHA}"
  echo "  2. git reset --hard ${TARGET}"
  echo "  3. pnpm install --frozen-lockfile"
  exit 0
fi

# Refuse to clobber uncommitted work on a real run.
if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree has uncommitted changes; commit or stash first." >&2
  echo "       run \`git status\` to see what's in flight." >&2
  exit 1
fi

echo "Step 1: saving backup branch ${BACKUP_BRANCH} at ${CURRENT_SHA}"
git branch "${BACKUP_BRANCH}" "${CURRENT_SHA}"

echo "Step 2: hard-resetting ${CURRENT_BRANCH} to ${TARGET}"
git reset --hard "${TARGET}"

echo "Step 3: reinstalling pnpm deps to match the rolled-back state"
pnpm install --frozen-lockfile

echo ""
echo "Done. ${CURRENT_BRANCH} is now at ${TARGET}."
echo "Your previous work is safe on branch ${BACKUP_BRANCH}."
echo "To restore: git reset --hard ${BACKUP_BRANCH}"
