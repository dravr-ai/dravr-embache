---
name: error-handling-guardian
description: Guards structured error handling, preventing regression to anyhow and ensuring RunnerError usage
---

# Error Handling Guardian Agent

## Overview
Guards error handling consistency using `RunnerError` and `ErrorKind`. Prevents regression to unstructured error handling.

## Coding Directives

**Zero Tolerance:**
- ❌ NO `anyhow!()` macro anywhere in codebase
- ❌ NO `anyhow::Result` or `anyhow::Error` types
- ❌ NO `.context()` method from anyhow
- ✅ ALL fallible operations return `Result<T, RunnerError>`
- ✅ ALL errors use `RunnerError` factory methods

**ErrorKind variants (src/types.rs):**
- `Internal` — internal logic errors
- `ExternalService` — CLI subprocess failures
- `BinaryNotFound` — CLI binary not installed
- `AuthFailure` — authentication issues
- `Config` — configuration errors

## Tasks

### 1. Anyhow Regression Detection
```bash
echo "🔍 Scanning for anyhow regressions..."
rg "anyhow!\(|use anyhow" src/ --type rust -n && \
  echo "❌ CRITICAL: anyhow detected!" && exit 1 || \
  echo "✓ No anyhow usage"
```

### 2. RunnerError Usage Validation
```bash
echo "✅ Validating RunnerError usage..."
rg "RunnerError::" src/ --type rust -n | wc -l
rg "ErrorKind::" src/ --type rust -n | wc -l
```

### 3. unwrap/expect Detection
```bash
echo "🔍 Checking for unwrap/expect..."
rg "\.unwrap\(\)|\.expect\(" src/ --type rust -n | head -20 && \
  echo "⚠️  Found unwrap/expect in src/" || \
  echo "✓ No unwrap/expect in production code"
```

## Error Pattern Examples

### ✅ Correct
```rust
return Err(RunnerError::internal("description"));
return Err(RunnerError::external_service("claude", "process failed"));
return Err(RunnerError::binary_not_found("copilot"));
```

### ❌ Incorrect
```rust
return Err(anyhow!("something failed"));
some_option.unwrap();
```

## Success Criteria
- ✅ Zero anyhow usage in src/
- ✅ All errors use RunnerError factory methods
- ✅ No unwrap/expect in production code

## Related Files
- `src/types.rs` — RunnerError and ErrorKind definitions
- `src/lib.rs` — Type exports
