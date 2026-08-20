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

## Publish an unsigned DMG on GitHub

The `unsigned macOS DMG` workflow builds an Apple Silicon DMG, creates or
updates the matching GitHub Release, and uploads both the DMG and its SHA-256
checksum. It does not use Apple certificates or repository secrets.

The Git tag must match the desktop version in `desktop/src-tauri/tauri.conf.json`.
For version `0.0.1-alpha.1`, publish with:

```sh
git tag v0.0.1-alpha.1
git push origin v0.0.1-alpha.1
```

The tag push starts `.github/workflows/release.yml`. The workflow can also be
run manually from the Actions page for an existing tag. Its arm64 output is
created under:

```text
desktop/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/
```

Build the same unsigned DMG locally on Apple Silicon with:

```sh
cd desktop
MOMONOGI_TARGET=aarch64-apple-darwin \
  pnpm tauri build --target aarch64-apple-darwin --bundles dmg --no-sign --ci
```

Because the DMG is unsigned and not notarized, macOS may require users to
Control-click Momonogi and choose Open on first launch. The release checksum
verifies the download but does not replace code signing.

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

The unsigned workflow is suitable for personal distribution and early testing.
A warning-free public release still requires an Apple Developer ID,
hardened-runtime signing, and notarization. Those credentials are intentionally
not stored in the repository.
