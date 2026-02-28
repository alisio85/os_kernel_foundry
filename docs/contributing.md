# Contributing

## Scope

Contributions are welcome, especially in these areas:

- improving portability of the trait abstractions
- adding more deterministic test helpers
- refining error types and ergonomics
- expanding documentation and examples

## Development setup

```bash
cargo build
cargo test
cargo fmt --all
cargo clippy --all-targets --all-features
```

## Guidelines

- Keep abstractions minimal and well-scoped.
- Prefer trait-based contracts over concrete implementations.
- Ensure new functionality is covered by unit tests.
- Avoid introducing heap allocation requirements in core types.

## Pull requests

A good PR includes:

- a clear problem statement
- changes limited to one concern
- tests demonstrating the behavior
- documentation updates when appropriate
