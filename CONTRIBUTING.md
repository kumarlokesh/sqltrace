# Contributing to SQLTrace

Thank you for your interest in contributing to SQLTrace! This document outlines the process for contributing to the project.

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Development Workflow

1. Fork the repository and create your branch from `main`
2. If you've added code that should be tested, add tests
3. If you've changed APIs, update the documentation
4. Ensure the test suite passes
5. Make sure your code lints
6. Issue a pull request

## RFC Process

For substantial changes, we follow an RFC (Request for Comments) process to ensure that changes are well thought out and have been discussed before implementation begins.

### When to create an RFC

You should create an RFC for:

- New features or significant functionality changes
- Major architectural changes
- Changes that affect backward compatibility
- Changes that require coordination across multiple components

### RFC Lifecycle

1. **Draft**: Initial proposal (PR with `[WIP]` prefix)
2. **Review**: Open for discussion (at least 3 days)
3. **Last Call**: Final comment period (at least 2 days)
4. **Final Comment Period (FCP)**: Final review by maintainers (1 week)
5. **Approved/Rejected**: Decision made

### Creating an RFC

1. Copy `docs/rfcs/0000-template.md` to `docs/rfcs/0000-my-feature.md` (where 'my-feature' is descriptive)
2. Fill in the RFC template
3. Submit a pull request with the RFC
4. Update the pull request with feedback until it gets approved
5. Once approved, the RFC will be merged and assigned a number

## Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run lints
cargo clippy

# Format code
cargo fmt
```

## Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` and `cargo clippy` before submitting PRs
- Document public APIs with Rustdoc comments

## License

By contributing to SQLTrace, you agree that your contributions will be licensed under its MIT OR Apache-2.0 license.
