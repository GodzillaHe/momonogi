# Momonogi

[English](README.md) | [简体中文](README.zh-CN.md)

Momonogi is a local, file-based shared memory system for multiple AI agents. It
ships with a Rust CLI named `momo` and an optional macOS desktop app.

- Agent roles are configurable per store. By default, Codex and Claude Code are
  equal writers while OpenCode and OpenClaw are readers.
- Markdown notes stay portable and inspectable.
- Kernel file locks serialize writes; ETags reject stale updates.
- `MEMORY.md` remains a small pointer index, so agents load detail only when it
  is relevant.
- Lifecycle hooks remind writers to reconcile durable work before compaction.

## Install

Rust 1.85 or newer is required.

```sh
cargo install --path . --locked --force
momo --version
```

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

## Manage access

Inspect the current roles and manifest ETag:

```sh
momo access list ~/.local/share/momonogi/store --json
```

A current writer can grant, change, or revoke any Agent role. Every mutation
requires the current manifest ETag, so two writers cannot silently overwrite
each other's access changes:

```sh
momo access grant ROOT opencode --role writer --by codex --if-match ETAG
momo access set ROOT openclaw --role reader --by codex --if-match ETAG
momo access revoke ROOT openclaw --by codex --if-match ETAG
```

`set` is an alias of `grant`. Momonogi rejects unauthorized actors, stale
ETags, invalid or duplicate Agent IDs, and any change that would leave the
store without a writer. A no-op assignment leaves the revision and ETag
unchanged.

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

`configure` reads the role manifest instead of assuming fixed host roles. A
writer receives write rules and managed lifecycle hooks; a reader receives
read-only rules and has Momonogi-managed hooks removed; an Agent absent from
the manifest receives a no-access rule. Rerun `configure` for affected hosts
after changing access.

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
| `momo access list [ROOT] [--json]` | Show roles, manifest revision, and ETag |
| `momo access grant ROOT ID --role ROLE --by WRITER --if-match ETAG` | Grant or change a role (`set` is an alias) |
| `momo access revoke ROOT ID --by WRITER --if-match ETAG` | Remove an Agent from the manifest |
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

## Desktop app

Momonogi Desktop discovers local Agent hosts, manages their store roles,
previews host rule and hook changes before applying them, and browses global
and registered project memories and tags. It bundles a matching `momo` sidecar
for generated lifecycle hooks. The interface supports English and Simplified
Chinese, follows the system language on first launch, and remembers later
language changes.

The current desktop prerelease is `0.0.1-alpha.2` for Apple Silicon Macs. Open
[GitHub Releases](https://github.com/GodzillaHe/momonogi/releases) and download:

- `Momonogi_0.0.1-alpha.2_aarch64.dmg`
- `Momonogi_0.0.1-alpha.2_aarch64.dmg.sha256`

Verify the download before opening it:

```sh
cd ~/Downloads
shasum -a 256 -c Momonogi_0.0.1-alpha.2_aarch64.dmg.sha256
```

Open the DMG and move `Momonogi.app` to `/Applications`. The app uses an ad-hoc
signature for bundle integrity. It has no Apple Developer ID signature or
notarization, so macOS may require Control-click and Open on first launch. If
Gatekeeper reports that the app is damaged after the checksum passes, remove
the quarantine attribute from the installed copy:

```sh
xattr -dr com.apple.quarantine /Applications/Momonogi.app
open /Applications/Momonogi.app
```

Build and deployment instructions are in
[docs/DESKTOP_DEPLOYMENT.md](docs/DESKTOP_DEPLOYMENT.md).

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

1. Clone the Momonogi repository and install the Rust binary with Cargo.
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
