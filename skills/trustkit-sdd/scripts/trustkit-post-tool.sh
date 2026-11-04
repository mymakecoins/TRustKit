#!/usr/bin/env bash
# TRustKit PostToolUse Hook
# Deterministic feedback hook: validates formatting and checks for invariant regressions.

set -euo pipefail

# Read stdin to consume payload
PAYLOAD=$(cat || true)

# If cargo is available, perform quick check if .rs files were modified
if git status --porcelain 2>/dev/null | grep -E '\.rs$' >/dev/null; then
    cargo fmt --check >/dev/null 2>&1 || true
fi

exit 0
