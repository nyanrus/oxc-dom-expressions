# Task Completion Checklist

When completing a coding task, follow these steps:

## 1. Build and Verify
```bash
# Ensure code compiles without errors
cargo build

# Check for warnings (should be minimal)
cargo build 2>&1 | grep warning
```

## 2. Format Code
```bash
# Format all code
cargo fmt

# Verify formatting
cargo fmt -- --check
```

## 3. Lint
```bash
# Run clippy to catch common issues
cargo clippy
```

## 4. Test
```bash
# Run all tests
cargo test

# Run specific test suites as needed
cargo test --lib
cargo test --test dom_fixtures
```

## 5. Review Changes
```bash
# Check git status
git status

# Review changes
git diff

# Check for unintended changes
git diff --stat
```

## 6. Documentation
- Update relevant documentation files if API changed
- Update README.md if user-facing features added
- Add doc comments to new public APIs

## 7. Benchmarks (if performance-critical)
```bash
# Run benchmarks if changes affect performance
cargo bench
```

## Expected Standards
- **No compilation errors**
- **Minimal warnings** (unused variables should have `_` prefix)
- **All tests pass** (or only expected failures documented)
- **Code formatted** with cargo fmt
- **No clippy warnings** (or documented why ignored)
- **Documentation updated** if needed
