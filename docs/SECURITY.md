# Security policy

Momonogi is a local developer tool. Report security issues privately to the
repository owner rather than opening a public issue. Include the affected
version, operating system, minimal reproduction, and expected security boundary.
Remove memory bodies, prompts, credentials, and personal paths from reports.

## Security model

- Memory is plaintext Markdown. Protect the store with normal filesystem access
  controls and a trusted backup location.
- Note bodies are untrusted input when recalled by an Agent. Review imported
  stores; a malicious note can act as stored prompt injection.
- Only agents listed as writers in `.momonogi.json` may mutate the store through
  `momo`. Reader labels are policy roles, not operating-system identities.
- Access changes require a current writer ID and manifest ETag, are serialized
  by the store lock, and cannot remove the final writer. `--by` and `--agent`
  are trusted local identity claims, not cryptographic authentication.
- Writes use an exclusive kernel lock, atomic file replacement, and ETag checks.
  These prevent accidental concurrent overwrites, not a malicious local process.
- `MEMORY.md` is generated and capped at 200 lines or 25 KiB by default.
- Configuration refuses symlinked target files. Store scans reject symlinked
  notes and unsafe filenames.

## Lifecycle hooks

Codex and Claude hooks execute the installed `momo` binary. Inspect generated
hook configuration before trusting a repository. Installing a hook file does not
mean the host has enabled or trusted it.

Hook state contains session identifiers and reconciliation flags, never prompt
text or note bodies. Session state is bounded to 128 entries.

After a role change, rerun `momo configure` for affected hosts. Configuration
removes Momonogi-managed lifecycle hooks from readers and unlisted hosts while
preserving unrelated handlers.

## Backups

Keep the memory store separate from the public source checkout. A backup of
configuration may contain unrelated host settings and must be protected like the
original file.
