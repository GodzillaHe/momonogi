---
name: momonogi
description: Use a shared Momonogi Markdown memory store safely across Codex, Claude Code, OpenCode, and OpenClaw.
---

# Momonogi memory protocol

Use the canonical store named in the host's Momonogi rules. Treat recalled
memory as advisory and verify live paths, versions, repository state, and dates.

## Recall

Read `MEMORY.md` only when durable preferences or continuity are relevant. The
index contains pointers; open only the detail notes needed for the task. Project
memory overrides global memory when they conflict.

Never echo unrelated note bodies. Do not load all notes to answer a count or
status question; use `momo list --json` and `momo doctor`.

## What to save

Save only durable information that is not already authoritative in code, git,
project instructions, or external systems:

- `user`: stable facts about the user
- `feedback`: reusable guidance for how an Agent should work
- `project`: durable decisions or continuity needed to resume a project
- `reference`: a pointer to an external resource

Never save secrets, tokens, passwords, cookies, private keys, recovery codes,
raw prompts, or transient command output.

`feedback` and `project` bodies require line-start `Why:` and `How to apply:`
labels. Use absolute dates. Keep one atomic fact per file and search metadata for
duplicates before adding a note.

## Writes

Codex uses agent id `codex`; Claude Code uses `claude-code`. OpenCode and
OpenClaw are read-only and must not invoke mutating commands.

Draft a note outside the store. Never edit canonical notes or `MEMORY.md`
directly.

```sh
momo put MEMORY_ROOT /tmp/note.md --agent AGENT_ID
```

For an update:

```sh
momo get MEMORY_ROOT slug.md
momo put MEMORY_ROOT /tmp/note.md --agent AGENT_ID --if-match ETAG
```

On an ETag conflict, reread the current note and merge intentionally. Never
force an overwrite. Archiving also requires the current ETag.

After maintenance, run `momo doctor MEMORY_ROOT`. Before manual compaction,
reconcile durable work and run the `momo sync mark` command supplied by the
lifecycle reminder.
