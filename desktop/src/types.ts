export type ViewId = "agents" | "memories" | "tags" | "settings";
export type AgentRole = "writer" | "reader" | "none";
export type ManagedHookState = "active" | "partial" | "missing" | "invalid" | "not-applicable";

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
  configured: boolean;
  managed: boolean;
  hookState: ManagedHookState;
  configPaths: string[];
  configIssue?: string;
}

export interface AgentDiscoveryPayload {
  agents: AgentSummary[];
  storeRoot: string;
  storeAvailable: boolean;
  storeRevision?: number;
  storeEtag?: string;
  storeIssue?: string;
}

export interface AccessUpdatePayload {
  changed: boolean;
  etag: string;
  revision: number;
  writers: string[];
  readers: string[];
}

export interface AccessUpdateInput {
  agentId: string;
  role: AgentRole;
  actor: string;
  ifMatch: string;
}

export type StoreKind = "global" | "project";
export type StoreHealth = "ready" | "missing" | "invalid";

export interface StoreSummary {
  kind: StoreKind;
  path: string;
  health: StoreHealth;
  storeId?: string;
  revision?: number;
  issue?: string;
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
