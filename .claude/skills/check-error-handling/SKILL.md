---
name: check-error-handling
description: Validates error handling follows RunnerError pattern, detects anyhow regression, ensures ErrorKind usage
user-invocable: true
---

# Check Error Handling Skill

## Purpose
Validates that error handling follows the RunnerError/ErrorKind pattern. Detects anyhow regression.

## Usage
Run this skill before committing error handling changes.

## Prerequisites
- ripgrep (`rg`)

## Commands

### Quick Check
```bash
echo "🔍 Checking for anyhow usage..."

# 1. Check imports (FORBIDDEN)
if rg "use anyhow::|use anyhow;" src/ --type rust --quiet; then
    echo "❌ FAIL: anyhow imports detected!"
    rg "use anyhow" src/ --type rust -n | head -10
    exit 1
else
    echo "✓ PASS: No anyhow imports"
fi

# 2. Check macro usage (FORBIDDEN)
if rg "anyhow!\(" src/ --type rust --quiet; then
    echo "❌ FAIL: anyhow! macro detected!"
    rg "anyhow!\(" src/ --type rust -n | head -10
    exit 1
else
    echo "✓ PASS: No anyhow! macro"
fi

# 3. Check RunnerError usage
RUNNER_ERROR_COUNT=$(rg "RunnerError::" src/ --type rust | wc -l)
echo "✓ RunnerError usage: $RUNNER_ERROR_COUNT occurrences"

# 4. Check ErrorKind usage
ERRORKIND_COUNT=$(rg "ErrorKind::" src/ --type rust | wc -l)
echo "✓ ErrorKind usage: $ERRORKIND_COUNT occurrences"

echo ""
echo "✅ Error handling check PASSED"
```

## Success Criteria
- ✅ Zero `use anyhow` imports in src/
- ✅ Zero `anyhow!()` macro usage
- ✅ RunnerError used throughout
- ✅ ErrorKind variants used for error classification

## Fixing Violations

### Replace anyhow
```rust
// ❌ Before
return Err(anyhow!("Binary not found"));

// ✅ After
return Err(RunnerError::binary_not_found("claude"));
return Err(RunnerError::internal("description"));
return Err(RunnerError::external_service("service", "description"));
```

## Related Files
- `src/types.rs` - RunnerError and ErrorKind definitions
- `src/lib.rs` - Type exports
