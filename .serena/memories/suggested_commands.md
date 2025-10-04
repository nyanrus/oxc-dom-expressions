# Suggested Commands

## Building
```bash
cargo build                  # Build the library
cargo build --release        # Build with optimizations
```

## Testing
```bash
cargo test                   # Run all tests
cargo test --test dom_fixtures            # Run DOM mode fixture tests
cargo test --test ssr_fixtures            # Run SSR mode fixture tests
cargo test --test hydratable_fixtures     # Run hydratable mode fixture tests
cargo test --test integration             # Run integration tests
```

## Linting and Formatting
```bash
cargo fmt                    # Format code
cargo clippy                 # Run linter
cargo clippy -- -D warnings  # Run linter with warnings as errors
```

## Benchmarking
```bash
cargo bench                  # Run all benchmarks
```

## Examples
```bash
cargo run --example debug_text_interpolation
```

## Development Workflow
1. Make code changes
2. Run `cargo fmt` to format
3. Run `cargo clippy` to check for issues
4. Run `cargo test` to ensure tests pass
5. Run `cargo bench` if performance-sensitive changes
