import { invoke } from "@tauri-apps/api/core";
import { agentCatalog } from "./agent-catalog";
import { mockAgentDiscovery } from "./mock-data";
import type { AgentDiscoveryPayload, BootstrapPayload } from "./types";

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

export async function getAgentDiscovery(): Promise<AgentDiscoveryPayload> {
  if (!isTauriRuntime()) {
    return mockAgentDiscovery;
  }

  const catalog = agentCatalog.map(({ id, name, commands }) => ({ id, name, commands }));
  return invoke<AgentDiscoveryPayload>("discover_agents", { catalog });
}
