# Publishing & releases

## crates.io

This crate is intended to be published to crates.io.

Typical local dry-run:

```bash
cargo publish --dry-run
```

## GitHub release flow

A common flow is:

1. Update `Cargo.toml` version
2. Update `Cargo.lock` if applicable
3. Tag a release (e.g. `v0.1.2`)
4. Push the tag

The repository includes a GitHub Actions workflow that can publish on tag pushes.

## Trusted Publishing

The included `publish.yml` uses Rust’s crates.io auth action (OIDC / trusted publishing) rather than a long-lived token.

Make sure your crates.io settings and GitHub repository settings are configured accordingly.
