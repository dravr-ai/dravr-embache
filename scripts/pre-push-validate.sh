#!/bin/bash
# ABOUTME: Pre-push validation script for embache
# ABOUTME: Runs fmt, clippy, tests and creates validation marker
#
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 dravr.ai

set -e

PROJECT_ROOT="$(git rev-parse --show-toplevel)"
GIT_DIR="$(git rev-parse --git-dir)"
MARKER_FILE="$GIT_DIR/validation-passed"

echo "🔍 Running pre-push validation..."
echo ""

# Tier 0: Format check
echo "━━━ Tier 0: Format Check ━━━"
if ! cargo fmt -- --check; then
    echo "❌ Format check failed. Run: cargo fmt"
    exit 1
fi
echo "✅ Format OK"
echo ""

# Tier 1: Clippy
echo "━━━ Tier 1: Clippy ━━━"
if ! cargo clippy --all-targets 2>&1; then
    echo "❌ Clippy failed"
    exit 1
fi
echo "✅ Clippy OK"
echo ""

# Tier 2: Tests
echo "━━━ Tier 2: Tests ━━━"
if ! cargo test 2>&1; then
    echo "❌ Tests failed"
    exit 1
fi
echo "✅ Tests OK"
echo ""

# Create validation marker
CURRENT_COMMIT=$(git rev-parse HEAD)
CURRENT_TIMESTAMP=$(date +%s)
echo "$CURRENT_TIMESTAMP $CURRENT_COMMIT" > "$MARKER_FILE"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All validation passed!"
echo "   Marker created: $MARKER_FILE"
echo "   You can now: git push"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
