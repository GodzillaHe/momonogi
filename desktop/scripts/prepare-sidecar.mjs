import { spawnSync } from "node:child_process";
import { chmodSync, copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptsDirectory = dirname(fileURLToPath(import.meta.url));
const desktopDirectory = resolve(scriptsDirectory, "..");
const repositoryRoot = resolve(desktopDirectory, "..");

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: repositoryRoot,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
  });
  if (result.status !== 0) {
    if (options.capture && result.stderr) process.stderr.write(result.stderr);
    process.exit(result.status ?? 1);
  }
  return result.stdout ?? "";
}

const rustVersion = run("rustc", ["-vV"], { capture: true });
const targetTriple = rustVersion.match(/^host:\s+(.+)$/m)?.[1]?.trim();
if (!targetTriple) {
  throw new Error("cannot determine the Rust host target triple");
}

run("cargo", [
  "build",
  "--manifest-path",
  resolve(repositoryRoot, "Cargo.toml"),
  "--locked",
  "--release",
  "--bin",
  "momo",
  "--target",
  targetTriple,
]);

const extension = targetTriple.includes("windows") ? ".exe" : "";
const source = resolve(repositoryRoot, "target", targetTriple, "release", `momo${extension}`);
const destination = resolve(desktopDirectory, "src-tauri", "binaries", `momo-${targetTriple}${extension}`);
mkdirSync(dirname(destination), { recursive: true });
copyFileSync(source, destination);
if (!extension) chmodSync(destination, 0o755);

const version = run(destination, ["--version"], { capture: true }).trim();
process.stdout.write(`Prepared ${version} for ${targetTriple}\n`);
