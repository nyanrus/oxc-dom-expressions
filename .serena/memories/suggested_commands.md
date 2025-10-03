# Suggested Commands for oxc-dom-expressions

## Building
```bash
# Build the project
cargo build

# Build in release mode
cargo build --release
```

## Testing
```bash
# Run all tests
cargo test

# Run only unit tests (lib tests)
cargo test --lib

# Run specific test suite
cargo test --test dom_fixtures
cargo test --test ssr_fixtures
cargo test --test hydratable_fixtures

# Run specific test with output
cargo test --test dom_fixtures test_simple_elements -- --nocapture

# Run all fixture tests
cargo test --test dom_fixtures --test ssr_fixtures --test hydratable_fixtures
```

## Code Quality
```bash
# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all
```

## Benchmarking
```bash
# Compile benchmarks
cargo bench --no-run

# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench simple_element
```

## Examples
```bash
# Run basic usage example
cargo run --example basic_usage

# Run phase 4 demo (optimization features)
cargo run --example phase4_demo
```

## Documentation
```bash
# Build documentation
cargo doc

# Build and open documentation
cargo doc --open
```

## Utility Commands (Linux)
- `git`: Version control
- `ls`, `cd`, `pwd`: File navigation
- `grep`, `find`: Text/file search
- `cat`, `less`, `head`, `tail`: File viewing
