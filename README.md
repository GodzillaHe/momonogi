# Momonogi

Momonogi is a local, file-based shared memory system for multiple AI agents. It
ships as one Rust binary named `momo`.

- Codex and Claude Code are equal, concurrent-safe writers.
- OpenCode and OpenClaw are read-only consumers.
- Markdown notes stay portable and inspectable.
- Kernel file locks serialize writes; ETags reject stale updates.
- `MEMORY.md` remains a small pointer index, so agents load detail only when it
  is relevant.
- Lifecycle hooks remind writers to reconcile durable work before compaction.

## Install

Rust 1.85 or newer is required.

```sh
cargo install --path tools/momonogi --locked --force
momo --version
```

When running from this directory, use `cargo install --path . --locked --force`.

Create a new global store:

```sh
momo init ~/.local/share/momonogi/store \
  --store-id global \
  --writer codex \
  --writer claude-code \
  --reader opencode \
  --reader openclaw
```

An older compatible Markdown store can be adopted in place:

```sh
momo migrate ~/.local/share/momonogi/store --agent codex
```

Migration preserves note bodies, adds multi-writer metadata where missing, and
regenerates the index.

## Configure agents

```sh
momo configure \
  --host codex \
  --host claude \
  --host opencode \
  --host openclaw \
  --codex-project /path/to/project \
  --openclaw-workspace /path/to/openclaw/workspace
```

`configure` manages marked blocks in the normal host rule files. Claude hooks
are global in `~/.claude/settings.json`. Codex hooks are project-scoped, so pass
each repository with `--codex-project`. Use `--no-hooks` to install rules only.
Existing unrelated rules and hook handlers are preserved.

OpenClaw workspace rules are host-conditional so a shared project-level
`AGENTS.md` cannot downgrade Codex from writer to reader.

## Commands

| Command | Purpose |
| --- | --- |
| `momo init ROOT ...` | Create a store and its role manifest |
| `momo migrate ROOT --agent ID` | Adopt a compatible existing store |
| `momo list [ROOT]` | List all active memory metadata; defaults to the global store |
| `momo list --json` | Emit metadata only, never note bodies |
| `momo get ROOT SLUG.md` | Return the current ETag |
| `momo get ROOT SLUG.md --content` | Read one note |
| `momo put ROOT FILE --agent ID` | Add a note |
| `momo put ... --if-match ETAG` | Update without overwriting a concurrent edit |
| `momo archive ROOT SLUG.md --agent ID --if-match ETAG` | Archive a note |
| `momo reindex ROOT --agent ID` | Regenerate `MEMORY.md` |
| `momo doctor ROOT` | Validate manifest, notes, index, and limits |
| `momo configure ...` | Install host rules and lifecycle hooks |
| `momo hook ...` | Lifecycle hook entry point |
| `momo sync status ROOT` | Inspect lifecycle reconciliation state |
| `momo sync mark ROOT --session-id ID` | Mark a session reconciled |
| `momo logo` | Print the Momonogi logo |

Run `momo COMMAND --help` for complete options.

For Agent-focused installation and trust checks, see
[docs/AGENT_SETUP.md](docs/AGENT_SETUP.md). The reusable Agent protocol lives in
[skill/SKILL.md](skill/SKILL.md).

## Write protocol

Draft notes outside the canonical store, then write through `momo`:

```markdown
---
name: Prefer concise progress updates
description: Keep intermediate updates short and specific
type: feedback
scope: global
created: 2026-08-20
updated: 2026-08-20
---

Keep progress updates concise.

Why: long updates interrupt the workflow.

How to apply: report the result, current risk, and next action in two sentences.
```

```sh
momo put ~/.local/share/momonogi/store /tmp/concise-updates.md --agent codex
```

For updates, fetch the ETag first and pass it with `--if-match`. Never edit
canonical notes or `MEMORY.md` directly.

## Moving to another computer

1. Clone Azusa and install the Rust binary with Cargo.
2. Copy or securely sync the memory store separately. It may contain personal
   data and should not be committed to a public repository.
3. Run `momo doctor ROOT` before configuration.
4. Run `momo configure` for the agents available on that computer.

The store contract is `.momonogi.json`, `MEMORY.md`, Markdown notes, and an
optional `archive/` directory. It is independent of the source directory after
installation.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Momonogi is MIT licensed. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
for attribution. Maintenance policies are in [docs/](docs/).
