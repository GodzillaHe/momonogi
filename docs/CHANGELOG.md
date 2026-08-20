# Changelog

## 1.0.0 - 2026-08-20

- Reimplemented the `momo` CLI and memory store in Rust.
- Added equal writer roles for Codex and Claude Code.
- Added read-only roles for OpenCode and OpenClaw.
- Added kernel file locking, atomic writes, ETags, and per-note revisions.
- Added global metadata listing, safe archiving, migration, reindexing, and doctor checks.
- Added idempotent host rule configuration and lifecycle hooks.
- Made OpenClaw workspace rules host-conditional so shared `AGENTS.md` files do
  not override Codex or other hosts' global roles.
- Preserved the existing Markdown store and index format.
