import { invoke } from "@tauri-apps/api/core";
import type { BootstrapPayload } from "./types";

const browserPayload: BootstrapPayload = {
  appVersion: "0.1.0-dev",
  coreSchema: 1,
  bridge: "browser",
};

function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function getBootstrap(): Promise<BootstrapPayload> {
  if (!isTauriRuntime()) {
    return browserPayload;
  }

  return invoke<BootstrapPayload>("bootstrap");
}
