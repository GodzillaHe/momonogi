export type ViewId = "agents" | "memories" | "tags" | "settings";
export type AgentRole = "writer" | "reader" | "none";

export interface BootstrapPayload {
  appVersion: string;
  coreSchema: number;
  bridge: "desktop" | "browser";
}

export interface AgentSummary {
  id: string;
  name: string;
  command: string;
  role: AgentRole;
  installed: boolean;
  managed: boolean;
}

export interface MemorySummary {
  slug: string;
  title: string;
  type: "user" | "feedback" | "project" | "reference";
  scope: "global" | "project";
  project?: string;
  updated: string;
  tags: string[];
  excerpt: string;
}

export interface TagSummary {
  name: string;
  count: number;
  scope: "global" | "mixed" | "project";
}
