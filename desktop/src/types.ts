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
  storeId: string;
  storeKind: StoreKind;
  storePath: string;
  slug: string;
  archived: boolean;
  name: string;
  description: string;
  memoryType: "user" | "feedback" | "project" | "reference";
  scope: string;
  status: string;
  updated: string;
  revision: number;
  tags: string[];
  etag: string;
}

export interface MemoryIssue {
  storePath: string;
  slug?: string;
  message: string;
}

export interface MemoryIndexPayload {
  notes: MemorySummary[];
  issues: MemoryIssue[];
}

export interface MemoryDetailPayload {
  summary: MemorySummary;
  body: string;
  content: string;
}

export interface TagSummary {
  name: string;
  count: number;
  scope: "global" | "mixed" | "project";
}
