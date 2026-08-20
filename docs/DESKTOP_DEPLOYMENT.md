# Momonogi Desktop Deployment

Momonogi Desktop is a local macOS application built with Tauri. The app bundle
contains a matching `momo` CLI sidecar so lifecycle hooks keep working after
the app is moved out of the source checkout.

## Build an arm64 app

Prerequisites:

- Apple Silicon Mac
- Rust 1.85 or newer
- Node.js 20 or newer
- pnpm 10 or newer

From a fresh clone:

```sh
cd desktop
pnpm install --frozen-lockfile
pnpm test
pnpm tauri build --bundles app
```

The Tauri pre-build step compiles the root `momo` binary for the active Rust
host target and stages it as an external binary. Generated sidecars are ignored
by Git. The resulting application is:

```text
desktop/src-tauri/target/release/bundle/macos/Momonogi.app
```

Verify both arm64 executables are present:

```sh
file src-tauri/target/release/bundle/macos/Momonogi.app/Contents/MacOS/momonogi-desktop
file src-tauri/target/release/bundle/macos/Momonogi.app/Contents/MacOS/momo
src-tauri/target/release/bundle/macos/Momonogi.app/Contents/MacOS/momo --version
```

## Install locally

Move `Momonogi.app` to `/Applications` and launch it. A locally built bundle is
unsigned; macOS may require the usual Control-click and Open flow. Do not remove
quarantine flags from an app obtained from someone you do not trust.

The embedded CLI is used by configuration previews and generated lifecycle
hooks. To use `momo` directly in a terminal, install the same source revision:

```sh
cargo install --path . --locked --force
```

## Move to another computer

1. Clone this repository and build the app on the destination architecture.
2. Install the CLI or keep the bundled sidecar inside `Momonogi.app`.
3. Copy or securely sync the memory store separately from the source tree.
4. Run `momo doctor ROOT` before registering the store.
5. Open Momonogi Desktop, register project stores, choose a current writer,
   update roles, inspect the configuration preview, and apply it explicitly.

Memory stores can contain personal information. Do not commit them to this
repository or package them inside the application.

## Release signing

The local build is suitable for personal deployment. A public release still
requires an Apple Developer ID, hardened-runtime signing, notarization, and a
published checksum. Those credentials are intentionally not stored in the
repository.
