#!/usr/bin/env bash
# Quick disk-usage report for the mizan-smart workspace.
#
# Usage:
#   pnpm disk:check
set -euo pipefail

cd "$(dirname "$0")/.."

show() {
  local label="$1"
  local path="$2"
  if [[ -e "$path" ]]; then
    printf "  %-22s %s\n" "$label" "$(du -sh "$path" 2>/dev/null | awk '{print $1}')"
  else
    printf "  %-22s -\n" "$label"
  fi
}

echo "Repo disk usage"
echo "----------------"
show "target/"               "target"
show "node_modules/"          "node_modules"
show "dist/"                  "dist"
show "build/"                 "build"
show ".git"                   ".git"
show "apps/tauri/gen/"        "apps/tauri/gen"
show "apps/tauri/src-tauri/target/" "apps/tauri/src-tauri/target"
show "playwright-report/"     "playwright-report"
show "test-results/"          "test-results"
echo ""

# Per-workspace node_modules (any local copies that shouldn't have appeared).
echo "Per-workspace node_modules"
echo "---------------------------"
find . -name "node_modules" -type d -prune 2>/dev/null \
  | grep -v "^./node_modules$" \
  | while read -r nm; do
      printf "  %-60s %s\n" "$nm" "$(du -sh "$nm" 2>/dev/null | awk '{print $1}')"
    done | head -20
echo ""

echo "Top-level total"
echo "----------------"
printf "  %-22s %s\n" "repo total" "$(du -sh . 2>/dev/null | awk '{print $1}')"
echo ""

# Disk-wide pressure.
echo "Volume free space"
echo "------------------"
df -h . | tail -2
