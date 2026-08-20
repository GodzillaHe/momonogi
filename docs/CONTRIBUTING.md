# Contributing

Momonogi is intentionally small. Prefer direct changes that preserve the file
format and concurrency contract over new layers or dependencies.

## Requirements

- Rust 1.85 or newer
- Cargo with the committed `Cargo.lock`

## Checks

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Tests should cover behavior through the `momo` binary where practical. Changes
to writes must exercise writer permissions, kernel locking, ETags, atomic index
replacement, and metadata-only JSON output. Host configuration tests must prove
idempotency and preservation of unrelated rules and hooks.

Never add real memory contents, machine-local credentials, absolute personal
paths, or generated `target/` output to the repository. Keep the English and
Chinese READMEs aligned for user-facing behavior.
