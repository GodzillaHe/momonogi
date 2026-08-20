import type { AgentSummary, MemorySummary, TagSummary } from "./types";

export const mockAgents: AgentSummary[] = [
  { id: "codex", name: "Codex", command: "codex", role: "writer", installed: true, managed: true },
  { id: "claude-code", name: "Claude Code", command: "claude", role: "writer", installed: true, managed: true },
  { id: "opencode", name: "OpenCode", command: "opencode", role: "reader", installed: true, managed: false },
  { id: "openclaw", name: "OpenClaw", command: "openclaw", role: "reader", installed: true, managed: true },
];

export const mockMemories: MemorySummary[] = [
  {
    slug: "agent-access-policy.md",
    title: "Agent access policy",
    type: "project",
    scope: "global",
    updated: "2026-08-20",
    tags: ["agents", "permissions"],
    excerpt: "Codex and Claude Code are equal writers. OpenCode and OpenClaw consume shared memory as readers.",
  },
  {
    slug: "momonogi-desktop.md",
    title: "Momonogi Desktop",
    type: "project",
    scope: "project",
    project: "momonogi",
    updated: "2026-08-20",
    tags: ["desktop", "tauri"],
    excerpt: "Desktop manager for Agent access, memory stores, search, and tags.",
  },
  {
    slug: "interface-preferences.md",
    title: "Interface preferences",
    type: "feedback",
    scope: "global",
    updated: "2026-08-18",
    tags: ["design", "workflow"],
    excerpt: "Operational tools should stay compact, direct, and easy to scan during repeated use.",
  },
];

export const mockTags: TagSummary[] = [
  { name: "agents", count: 4, scope: "global" },
  { name: "permissions", count: 3, scope: "global" },
  { name: "desktop", count: 2, scope: "project" },
  { name: "workflow", count: 2, scope: "mixed" },
  { name: "design", count: 1, scope: "global" },
  { name: "tauri", count: 1, scope: "project" },
];
