# Changelog

## 1.1.0 - 2026-08-20

- Added revisioned `momo access list`, `grant`/`set`, and `revoke` commands.
- Added writer authorization, manifest ETags, atomic access updates, and
  final-writer protection.
- Made host rules and lifecycle hooks follow each Agent's configured role.
- Added writer, reader, and no-access OpenClaw policies without weakening the
  host-conditional shared-workspace guard.
- Kept schema version 1 compatibility with existing manifests.

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
