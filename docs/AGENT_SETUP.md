# Agent setup

This runbook configures an existing Momonogi installation without reading or
rewriting memory bodies.

## 1. Verify the binary and store

```sh
momo --version
momo doctor ~/.local/share/momonogi/store
momo list ~/.local/share/momonogi/store --json
```

`list --json` contains metadata only. Stop if `doctor` fails; do not repair or
migrate a store without confirming its ownership and intended writers.

## 2. Confirm roles

The normal global manifest is:

- writers: `codex`, `claude-code`
- readers: `opencode`, `openclaw`

Only configured writers may run `put`, `archive`, `reindex`, or migration.
Readers must never edit, move, or copy the shared store into another memory
system.

## 3. Configure hosts

```sh
momo configure \
  --host codex \
  --host claude \
  --host opencode \
  --host openclaw \
  --memory-root ~/.local/share/momonogi/store \
  --codex-project /path/to/project \
  --openclaw-workspace /path/to/openclaw/workspace
```

Claude lifecycle hooks are global. Codex lifecycle hooks are project-scoped;
pass each trusted repository separately. Project hooks execute local commands,
so inspect `.codex/hooks.json` and approve them through the host's trust UI.

`configure` is idempotent. It replaces one marked Momonogi rules block and one
managed handler per lifecycle event while preserving unrelated content. It
refuses malformed JSON and symlinked configuration files.

## 4. Verify configuration

Confirm each host has the expected role text and that lifecycle handlers point
to the installed `momo` binary. Start a fresh Agent session after configuration;
an already-running session may still have the previous instructions in context.

Do not report hooks as active until the host displays or executes them. A file
being present proves configuration, not host trust or runtime activation.
