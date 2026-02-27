#!/usr/bin/env bash
# ABOUTME: Pre-push validation script - runs all checks before pushing
# ABOUTME: Creates validation-passed marker in git dir (supports worktrees)
#
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 dravr.ai

set -e

PROJECT_ROOT="$(git rev-parse --show-toplevel)"
GIT_DIR="$(git rev-parse --git-dir)"
MARKER_FILE="$GIT_DIR/validation-passed"
VALIDATION_TTL_MINUTES=15

echo ""
echo "🔍 Embacle - Pre-Push Validation"
echo "================================="
echo ""

START_TIME=$(date +%s)

# Remove any stale marker
rm -f "$MARKER_FILE"

# ============================================================================
# TIER 0: Code Formatting
# ============================================================================
echo "🎨 Tier 0: Code Formatting"
echo "--------------------------"
echo -n "Checking cargo fmt... "

if cargo fmt --all -- --check > /dev/null 2>&1; then
    echo "✅"
else
    echo "❌"
    echo ""
    echo "Code is not properly formatted. Run:"
    echo "  cargo fmt --all"
    exit 1
fi
echo ""

# ============================================================================
# TIER 1: Clippy (default features)
# ============================================================================
echo "📎 Tier 1: Clippy (default features)"
echo "-------------------------------------"
echo -n "Running clippy... "

if cargo clippy --all-targets --quiet 2>&1 | grep -q "^error"; then
    echo "❌"
    echo ""
    cargo clippy --all-targets 2>&1 | head -40
    exit 1
fi
echo "✅"
echo ""

# ============================================================================
# TIER 2: Clippy (copilot-sdk feature)
# ============================================================================
echo "📎 Tier 2: Clippy (copilot-sdk feature)"
echo "----------------------------------------"
echo -n "Running clippy --features copilot-sdk... "

if cargo clippy --all-targets --features copilot-sdk --quiet 2>&1 | grep -q "^error"; then
    echo "❌"
    echo ""
    cargo clippy --all-targets --features copilot-sdk 2>&1 | head -40
    exit 1
fi
echo "✅"
echo ""

# ============================================================================
# TIER 3: Tests (all features)
# ============================================================================
echo "🧪 Tier 3: Tests"
echo "-----------------"
echo -n "Running cargo test --lib --features copilot-sdk... "

if cargo test --lib --features copilot-sdk --quiet 2>&1; then
    echo "✅"
else
    echo "❌"
    echo ""
    echo "Tests failed. Run: cargo test --lib --features copilot-sdk -- --nocapture"
    exit 1
fi
echo ""

# ============================================================================
# SUCCESS - Create marker file
# ============================================================================
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Create marker with timestamp and commit hash
CURRENT_COMMIT=$(git rev-parse HEAD)
echo "$END_TIME $CURRENT_COMMIT" > "$MARKER_FILE"

echo "================================="
echo "✅ All validations passed!"
echo "================================="
echo ""
echo "Duration: ${DURATION}s (~$((DURATION / 60))m $((DURATION % 60))s)"
echo "Marker:   .git/validation-passed (valid for ${VALIDATION_TTL_MINUTES} minutes)"
echo ""
echo "You can now push:"
echo "  git push"
echo ""
