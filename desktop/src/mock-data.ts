import type { AgentDiscoveryPayload, AgentSummary, MemorySummary, StoreSummary } from "./types";

export const mockAgents: AgentSummary[] = [
  { id: "codex", name: "Codex", command: "codex", role: "writer", installed: true, configured: true, managed: true, hookState: "active", configPaths: ["~/.codex/AGENTS.md", "./.codex/hooks.json"] },
  { id: "claude-code", name: "Claude Code", command: "claude", role: "writer", installed: true, configured: true, managed: true, hookState: "active", configPaths: ["~/.claude/CLAUDE.md", "~/.claude/settings.json"] },
  { id: "opencode", name: "OpenCode", command: "opencode", role: "reader", installed: true, configured: true, managed: false, hookState: "not-applicable", configPaths: ["~/.config/opencode/AGENTS.md"] },
  { id: "openclaw", name: "OpenClaw", command: "openclaw", role: "reader", installed: true, configured: true, managed: true, hookState: "not-applicable", configPaths: ["~/Documents/openclaw/AGENTS.md"] },
];

export const mockAgentDiscovery: AgentDiscoveryPayload = {
  agents: mockAgents,
  storeRoot: "~/.local/share/momonogi/store",
  storeAvailable: true,
  storeRevision: 24,
  storeEtag: "browser-etag-24",
};

export const mockStores: StoreSummary[] = [
  { kind: "global", path: "~/.local/share/momonogi/store", health: "ready", storeId: "global", revision: 24 },
  { kind: "project", path: "~/Code/momonogi/.momonogi", health: "ready", storeId: "momonogi", revision: 8 },
];

export const mockMemories: MemorySummary[] = [
  {
    storeId: "global",
    storeKind: "global",
    storePath: "~/.local/share/momonogi/store",
    slug: "agent-access-policy.md",
    archived: false,
    name: "Agent access policy",
    description: "Equal writers and configurable readers",
    memoryType: "project",
    scope: "global",
    status: "active",
    updated: "2026-08-20",
    revision: 4,
    tags: ["agents", "permissions"],
    etag: "mock-agent-access",
  },
  {
    storeId: "momonogi",
    storeKind: "project",
    storePath: "~/Code/momonogi/.momonogi",
    slug: "momonogi-desktop.md",
    archived: false,
    name: "Momonogi Desktop",
    description: "Desktop manager for shared memory",
    memoryType: "project",
    scope: "repo",
    status: "active",
    updated: "2026-08-20",
    revision: 8,
    tags: ["desktop", "tauri"],
    etag: "mock-desktop",
  },
  {
    storeId: "global",
    storeKind: "global",
    storePath: "~/.local/share/momonogi/store",
    slug: "interface-preferences.md",
    archived: false,
    name: "Interface preferences",
    description: "Compact operational interfaces",
    memoryType: "feedback",
    scope: "global",
    status: "active",
    updated: "2026-08-18",
    revision: 2,
    tags: ["design", "workflow"],
    etag: "mock-interface",
  },
  {
    storeId: "momonogi",
    storeKind: "project",
    storePath: "~/Code/momonogi/.momonogi",
    slug: "old-layout.md",
    archived: true,
    name: "Old layout decision",
    description: "Archived desktop layout direction",
    memoryType: "feedback",
    scope: "repo",
    status: "archived",
    updated: "2026-08-10",
    revision: 3,
    tags: ["design", "legacy"],
    etag: "mock-old-layout",
  },
];

export const mockMemoryBodies: Record<string, string> = {
  "global:agent-access-policy.md": "Codex and Claude Code are equal writers. OpenCode and OpenClaw consume shared memory according to the active manifest.\n\nWhy: Agent permissions must remain explicit and reversible.\n\nHow to apply: change roles through Momonogi with a current writer identity.",
  "momonogi:momonogi-desktop.md": "Momonogi Desktop manages Agent access, registered stores, search, and tags.\n\nWhy: local memory operations need a compact control surface.\n\nHow to apply: keep the CLI as the automation interface and use the desktop app for inspection.",
  "global:interface-preferences.md": "Operational tools should stay compact, direct, and easy to scan during repeated use.\n\nWhy: dense workflows benefit from predictable information placement.\n\nHow to apply: use rows, hairlines, and restrained status signals.",
  "momonogi:old-layout.md": "The earlier layout direction has been archived.\n\nWhy: it no longer matches the operational workbench.\n\nHow to apply: keep it for historical context only.",
};
