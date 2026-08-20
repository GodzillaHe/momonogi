import { invoke } from "@tauri-apps/api/core";
import { agentCatalog } from "./agent-catalog";
import { mockAgentDiscovery, mockMemories, mockMemoryBodies, mockStores } from "./mock-data";
import type { AccessUpdateInput, AccessUpdatePayload, AgentDiscoveryPayload, BootstrapPayload, MemoryDetailPayload, MemoryIndexPayload, StoreSummary, TagMutationPayload } from "./types";

const browserPayload: BootstrapPayload = {
  appVersion: "0.1.0-dev",
  coreSchema: 1,
  bridge: "browser",
};

let browserDiscovery = structuredClone(mockAgentDiscovery);
let browserStores = structuredClone(mockStores);
let browserMemories = structuredClone(mockMemories);

export function resetBrowserBridgeForTests(): void {
  browserDiscovery = structuredClone(mockAgentDiscovery);
  browserStores = structuredClone(mockStores);
  browserMemories = structuredClone(mockMemories);
}

export async function getStoreRegistry(): Promise<StoreSummary[]> {
  if (isTauriRuntime()) {
    return invoke<StoreSummary[]>("get_store_registry");
  }
  return structuredClone(browserStores);
}

export async function registerProjectStore(projectPath: string): Promise<StoreSummary[]> {
  if (isTauriRuntime()) {
    return invoke<StoreSummary[]>("register_project_store", { projectPath });
  }
  const path = projectPath.trim();
  if (!path) throw new Error("project store path is required");
  if (browserStores.some((store) => store.path === path)) {
    throw new Error(`project store is already registered: ${path}`);
  }
  const segments = path.split(/[\\/]/).filter(Boolean);
  const leaf = segments.at(-1);
  const name = leaf === ".momonogi" ? (segments.at(-2) ?? "project") : (leaf ?? "project");
  browserStores.push({ kind: "project", path, health: "ready", storeId: name, revision: 1 });
  return structuredClone(browserStores);
}

export async function removeProjectStore(projectPath: string): Promise<StoreSummary[]> {
  if (isTauriRuntime()) {
    return invoke<StoreSummary[]>("remove_project_store", { projectPath });
  }
  browserStores = browserStores.filter((store) => store.kind === "global" || store.path !== projectPath);
  return structuredClone(browserStores);
}

export async function getMemoryIndex(): Promise<MemoryIndexPayload> {
  if (isTauriRuntime()) {
    return invoke<MemoryIndexPayload>("get_memory_index", {
      filter: { search: "", memoryTypes: [], statuses: [], scopes: [], archive: "all" },
    });
  }
  return { notes: structuredClone(browserMemories), issues: [] };
}

export async function getMemoryDetail(
  storePath: string,
  slug: string,
  archived: boolean,
): Promise<MemoryDetailPayload> {
  if (isTauriRuntime()) {
    return invoke<MemoryDetailPayload>("get_memory_detail", { storePath, slug, archived });
  }
  const summary = browserMemories.find(
    (memory) => memory.storePath === storePath && memory.slug === slug && memory.archived === archived,
  );
  if (!summary) throw new Error(`memory not found: ${slug}`);
  const body = mockMemoryBodies[`${summary.storeId}:${summary.slug}`] ?? "";
  return {
    summary: structuredClone(summary),
    writers: ["claude-code", "codex"],
    body,
    content: `---\nname: ${summary.name}\n---\n\n${body}\n`,
  };
}

export async function changeMemoryTag(input: {
  storePath: string;
  slug: string;
  tag: string;
  action: "add" | "remove";
  actor: string;
  ifMatch: string;
}): Promise<TagMutationPayload> {
  if (isTauriRuntime()) {
    return invoke<TagMutationPayload>("change_memory_tag", input);
  }
  const memory = browserMemories.find(
    (item) => item.storePath === input.storePath && item.slug === input.slug && !item.archived,
  );
  if (!memory) throw new Error(`memory not found: ${input.slug}`);
  if (!["claude-code", "codex"].includes(input.actor)) {
    throw new Error(`agent ${JSON.stringify(input.actor)} is not a configured writer`);
  }
  if (memory.etag !== input.ifMatch) throw new Error("etag conflict");
  const normalized = input.tag.trim().toLowerCase().replace(/\s+/g, "-");
  if (!/^[a-z0-9][a-z0-9._-]{0,31}$/.test(normalized)) throw new Error(`invalid tag ${JSON.stringify(input.tag)}`);
  const tags = new Set(memory.tags);
  const before = tags.size;
  if (input.action === "add") tags.add(normalized);
  else tags.delete(normalized);
  const changed = tags.size !== before;
  if (changed) {
    memory.tags = [...tags].sort();
    memory.revision += 1;
    memory.etag = `mock-${memory.slug}-${memory.revision}`;
  }
  return {
    changed,
    slug: memory.slug,
    tag: normalized,
    tags: [...memory.tags],
    revision: memory.revision,
    etag: memory.etag,
    indexLines: browserMemories.length,
    indexBytes: 1024,
  };
}

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
    return structuredClone(browserDiscovery);
  }

  const catalog = agentCatalog.map(({ id, name, commands }) => ({ id, name, commands }));
  return invoke<AgentDiscoveryPayload>("discover_agents", { catalog });
}

export async function setAgentAccess(input: AccessUpdateInput): Promise<AccessUpdatePayload> {
  if (isTauriRuntime()) {
    return invoke<AccessUpdatePayload>("set_agent_access", {
      agentId: input.agentId,
      role: input.role === "none" ? null : input.role,
      actor: input.actor,
      ifMatch: input.ifMatch,
    });
  }

  if (!browserDiscovery.storeEtag || input.ifMatch !== browserDiscovery.storeEtag) {
    throw new Error("manifest etag conflict");
  }
  const actor = browserDiscovery.agents.find((agent) => agent.id === input.actor);
  if (actor?.role !== "writer") {
    throw new Error(`agent ${JSON.stringify(input.actor)} is not a configured writer`);
  }
  const target = browserDiscovery.agents.find((agent) => agent.id === input.agentId);
  if (!target) {
    throw new Error(`unknown agent ${JSON.stringify(input.agentId)}`);
  }
  const writerCount = browserDiscovery.agents.filter((agent) => agent.role === "writer").length;
  if (target.role === "writer" && writerCount === 1 && input.role !== "writer") {
    throw new Error(`cannot ${input.role === "reader" ? "downgrade" : "revoke"} the final writer`);
  }
  const changed = target.role !== input.role;
  if (changed) {
    target.role = input.role;
    browserDiscovery.storeRevision = (browserDiscovery.storeRevision ?? 0) + 1;
    browserDiscovery.storeEtag = `browser-etag-${browserDiscovery.storeRevision}`;
  }
  return {
    changed,
    etag: browserDiscovery.storeEtag,
    revision: browserDiscovery.storeRevision ?? 0,
    writers: browserDiscovery.agents.filter((agent) => agent.role === "writer").map((agent) => agent.id),
    readers: browserDiscovery.agents.filter((agent) => agent.role === "reader").map((agent) => agent.id),
  };
}
