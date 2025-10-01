# Contributing to oxc-dom-expressions

Thank you for your interest in contributing to oxc-dom-expressions! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.88.0 or later
- Cargo (comes with Rust)

### Clone the Repository

```bash
git clone https://github.com/nyanrus/oxc-dom-expressions.git
cd oxc-dom-expressions
```

### Build the Project

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Examples

```bash
cargo run --example basic_usage
```

## Project Structure

```
oxc-dom-expressions/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── options.rs      # Configuration options
│   ├── transform.rs    # Main transformer implementation
│   ├── utils.rs        # Utility functions
│   └── tests.rs        # Integration tests
├── examples/           # Example usage
├── Cargo.toml          # Project manifest
├── README.md           # Project documentation
├── LICENSE             # MIT License
└── CONTRIBUTING.md     # This file
```

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue on GitHub with:
- A clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Your environment (OS, Rust version, etc.)

### Suggesting Features

Feature requests are welcome! Please open an issue describing:
- The feature you'd like to see
- Why it would be useful
- How it might work

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Ensure the code builds without warnings (`cargo build`)
7. Format your code (`cargo fmt`)
8. Run clippy for lints (`cargo clippy`)
9. Commit your changes (`git commit -m 'Add amazing feature'`)
10. Push to your branch (`git push origin feature/amazing-feature`)
11. Open a Pull Request

## Development Guidelines

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Add documentation comments for public APIs
- Keep functions focused and single-purpose

### Testing

- Write unit tests for new functions
- Add integration tests for new features
- Ensure all tests pass before submitting PR
- Aim for good test coverage

### Documentation

- Update README.md if adding user-facing features
- Add doc comments to public functions and types
- Update examples if API changes
- Keep CHANGELOG.md updated (if we add one)

## Implementation Roadmap

The following features are planned for implementation:

### Phase 1: Core Infrastructure ✅
- [x] Basic project structure
- [x] Configuration options
- [x] Utility functions
- [x] Traverse hooks

### Phase 2: AST Transformation (TODO)
- [ ] Template string generation
- [ ] Element cloning code generation
- [ ] Property/attribute setters
- [ ] Dynamic expression wrapping

### Phase 3: Advanced Features (TODO)
- [ ] Event delegation
- [ ] Special bindings (ref, classList, style)
- [ ] Component handling
- [ ] Fragment support
- [ ] Import injection
- [ ] SSR mode

### Phase 4: Optimization (TODO)
- [ ] Template deduplication
- [ ] Static analysis improvements
- [ ] Performance benchmarks

## Getting Help

If you need help:
- Open a discussion on GitHub
- Check existing issues and PRs
- Review the babel-plugin-jsx-dom-expressions documentation for reference

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the golden rule: treat others as you'd like to be treated

## License

By contributing to oxc-dom-expressions, you agree that your contributions will be licensed under the MIT License.
