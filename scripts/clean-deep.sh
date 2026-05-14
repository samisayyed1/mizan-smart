#!/usr/bin/env bash
# Deep clean of the mizan-smart workspace.
#
# Wipes every regeneratable artifact:
#   - the Rust workspace target/ (~20-30 GiB on a populated dev env)
#   - all node_modules dirs (root + every workspace)
#   - frontend build outputs (dist/, build/, .turbo/, .next/, .vite/)
#   - Tauri build outputs (apps/tauri/src-tauri/target, apps/tauri/gen)
#   - test artifacts (coverage/, playwright-report/, test-results/)
#
# pnpm/cargo will repopulate what's needed on the next build. Pass
# `--no-install` to skip the pnpm reinstall (useful when you just want
# to reclaim space and reinstall later).
#
# Usage:
#   pnpm clean:deep
#   pnpm clean:deep --no-install
set -euo pipefail

cd "$(dirname "$0")/.."

NO_INSTALL=0
for arg in "$@"; do
  case "$arg" in
    --no-install) NO_INSTALL=1 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

bytes_before=$(df -k . | tail -1 | awk '{print $4}')

echo "==> cargo clean (workspace target/)"
if command -v cargo >/dev/null 2>&1; then
  cargo clean 2>&1 | tail -3 || true
else
  echo "    cargo not found; skipping"
fi

echo "==> removing all node_modules"
find . -name "node_modules" -type d -prune -exec rm -rf {} + 2>/dev/null || true

echo "==> removing build artifacts"
rm -rf \
  dist \
  build \
  .turbo \
  .next \
  .vite \
  .parcel-cache \
  apps/tauri/src-tauri/target \
  apps/tauri/gen/schemas

echo "==> removing test artifacts"
rm -rf \
  coverage \
  .nyc_output \
  playwright-report \
  test-results \
  blob-report

echo "==> removing tsconfig build info"
find . -name "*.tsbuildinfo" -type f -delete 2>/dev/null || true

if [[ $NO_INSTALL -eq 0 ]]; then
  echo "==> pnpm install (frozen lockfile)"
  pnpm install --frozen-lockfile
else
  echo "==> skipping pnpm install (--no-install)"
fi

bytes_after=$(df -k . | tail -1 | awk '{print $4}')
reclaimed_kb=$(( bytes_after - bytes_before ))
if [[ $reclaimed_kb -gt 0 ]]; then
  echo ""
  echo "==> reclaimed $(( reclaimed_kb / 1024 )) MiB ($(( reclaimed_kb / 1024 / 1024 )) GiB)"
else
  # Negative means the disk is fuller now (pnpm restored deps). Show
  # the delta honestly so users aren't confused.
  echo ""
  echo "==> disk delta: $(( -reclaimed_kb / 1024 )) MiB used (pnpm restored deps)"
fi
