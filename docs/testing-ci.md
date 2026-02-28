# Testing & CI

## Testing philosophy

The crate is designed so that a large portion of kernel logic can be validated on the host:

- boot pipelines can be executed with mock `Architecture` implementations
- memory/device/scheduler/ipc contracts can be tested in isolation
- tests are meant to be deterministic and side-effect free

## Running tests

From the repository root:

```bash
cargo test
```

## Formatting

This repository uses `rustfmt`.

To check formatting:

```bash
cargo fmt --all -- --check
```

To format files:

```bash
cargo fmt --all
```

## Linting

The repository uses Clippy.

```bash
cargo clippy --all-targets --all-features
```

## GitHub Actions

The CI workflow typically runs:

- `cargo build`
- `cargo test`
- `cargo clippy --all-targets --all-features -- ...`
- `cargo fmt --all -- --check`

If CI fails while local builds succeed, compare the exact commands and the toolchain version used in CI.
