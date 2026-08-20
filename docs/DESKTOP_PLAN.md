# Momonogi Desktop Plan

## Objective

Build a local desktop control surface for Momonogi without replacing the
`momo` CLI. The desktop app should make three jobs fast and safe:

1. Discover supported Agent hosts and inspect their Momonogi configuration.
2. Manage writer, reader, and no-access roles through the existing manifest
   lock and ETag protocol.
3. Browse global and project memory stores, including metadata, bodies,
   archives, and tags.

## V1 Scope

### Included

- macOS arm64 desktop build with Tauri 2.
- React, TypeScript, and Vite frontend.
- Known host discovery for Codex, Claude Code, OpenCode, and OpenClaw.
- Generic display of arbitrary Agent IDs found in a store manifest.
- Role changes with writer authorization, ETag conflict handling, and
  final-writer protection.
- Global store discovery plus explicitly registered project stores.
- Store health, note counts, index limits, and manifest revision.
- Memory metadata search, type/status/scope/tag filters, and detail reading.
- Tag add/remove operations with note ETags and revisions.
- Host rule and managed lifecycle hook refresh after role changes.
- A hand-built peach application icon and a restrained Momonogi design system.

### Deferred

- Full Markdown body editing.
- Automatic scanning of the entire home directory or every mounted disk.
- Cloud sync, accounts, remote access, or multi-user authentication.
- Treating Agent IDs as cryptographic identities.
- Windows and Linux installers. The Rust core remains portable and those
  packages can follow after the macOS workflow is stable.

## Repository Shape

```text
momonogi/
|- src/
|  |- lib.rs                 # reusable Momonogi core
|  `- main.rs                # momo CLI adapter
|- desktop/
|  |- src/                   # React application
|  |- src-tauri/             # Tauri commands and packaging
|  |- package.json
|  `- vite.config.ts
|- tokens.css                # portable Hallmark design tokens
`- docs/DESKTOP_PLAN.md
```

The Tauri crate depends on the root `momonogi` library. It does not shell out
to `momo` for normal operations. The CLI and desktop app therefore share the
same validation, locking, ETag, and atomic-write behavior.

## Data Boundaries

- The canonical store remains Markdown plus `.momonogi.json`.
- UI reads and writes go through Rust APIs only.
- Tests use temporary stores and temporary home directories.
- The real global store is used only for explicit read-only smoke tests.
- Project stores are registered explicitly. Discovery may suggest stores from
  known project roots, but never performs an unbounded disk scan.
- Host configuration changes show affected files before they are applied.
- Unrelated host rules and hooks are never removed.

## Delivery Stages

### 1. Core library boundary

- Add `src/lib.rs` and consume it from the CLI.
- Keep all existing commands and output compatible.

Gate:

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --locked --offline
```

### 2. Desktop shell and design system

- Scaffold Tauri 2, React, TypeScript, and Vite.
- Add Workbench layout, N3 side rail, Ft2 status line, tokens, and peach icon.
- Provide a web-safe mock bridge for visual development and tests.

Gate: TypeScript check, frontend unit tests, Tauri compile, and screenshots at
320, 375, 414, 768, 1280, and 1440 CSS pixels.

### 3. Agent discovery

- Detect known binaries and canonical configuration paths.
- Report installed, configured, role, and managed-hook state.
- Show arbitrary manifest-only Agent IDs as generic entries.

Gate: adapter tests against temporary home layouts for all four known hosts.

### 4. Access matrix

- Render writer, reader, and no-access as a segmented control.
- Require a current writer actor before saving.
- Refresh on stale ETag and preserve at least one writer.

Gate: permission integration tests for grant, downgrade, revoke, no-op,
unauthorized actor, stale ETag, and final-writer protection.

### 5. Store registry

- Register the default global store and explicit project stores.
- Persist registry settings outside memory stores.
- Show store health and allow safe removal from the registry only.

Gate: registry persistence, duplicate-path, missing-path, and invalid-manifest
tests. Removing a registry entry must not remove store data.

### 6. Memory explorer

- Group stores into Global and Projects.
- Search metadata and tags; filter by type, state, scope, and archive status.
- Read a complete note in a detail pane without writing it.

Gate: search/filter tests plus archived-note and malformed-note error states.

### 7. Tags

- Define normalized, unique tags in note frontmatter.
- Add Rust API and `momo tag list/add/remove` commands.
- Expose tag editing in the detail pane with ETag protection.

Gate: parsing, rendering, normalization, duplicate, stale ETag, reader denial,
revision, index, and UI interaction tests.

### 8. Configuration synchronization

- Preview and apply host rule changes after access updates.
- Install hooks for writers and remove only Momonogi hooks for other roles.
- Keep OpenClaw workspace rules host-conditional.

Gate: temporary-host configuration tests and idempotence hashes.

### 9. Release readiness

- Run the Hallmark slop test and accessibility checks.
- Exercise keyboard-only navigation and reduced motion.
- Run a read-only smoke test against the real global store.
- Produce a macOS arm64 application bundle and deployment documentation.

Gate: all Rust/frontend/Tauri tests, production build, visual screenshots, and
an unchanged real store ETag after smoke testing.

## Design Direction

- Genre: modern-minimal developer tool.
- Macrostructure: Workbench.
- Navigation: N3 left side rail, converted to a compact top bar on narrow
  windows.
- Footer: Ft2 inline status line.
- Custom vibe: technical restraint, peach-bright, tactile, local-first.
- Brand anchor: `#ff47ad`, represented by an accessible OKLCH signal token.
- Display: Bricolage Grotesque. Body: IBM Plex Sans. Data: JetBrains Mono.
- Accent appears only in brand, active selection, focus, and explicit commands.
- Motion is limited to selection movement and detail-pane opacity.

The app is an operational tool, not a landing page. It uses dense rows,
hairlines, stable controls, and one level of panels. It does not use nested
cards, decorative gradients, oversized hero text, or explanatory marketing
copy.

## Commit Discipline

Each delivery stage receives its own commit after its gate passes. A stage is
not marked complete because code exists; it is complete only when its focused
tests and the relevant regression suite pass.
