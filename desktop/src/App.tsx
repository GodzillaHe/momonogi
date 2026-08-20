import {
  Brain,
  Check,
  ChevronRight,
  CircleAlert,
  Database,
  FolderPlus,
  FolderOpen,
  RefreshCw,
  Search,
  Settings,
  ShieldCheck,
  Tags,
  Trash2,
  UserRoundCog,
  X,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { AgentLogo } from "./AgentLogo";
import {
  getAgentDiscovery,
  getBootstrap,
  getMemoryDetail,
  getMemoryIndex,
  getStoreRegistry,
  registerProjectStore,
  removeProjectStore,
  setAgentAccess,
} from "./bridge";
import markUrl from "./assets/momonogi-mark.svg";
import { mockTags } from "./mock-data";
import type { AgentDiscoveryPayload, AgentRole, AgentSummary, BootstrapPayload, ManagedHookState, MemoryDetailPayload, MemoryIndexPayload, MemorySummary, StoreSummary, ViewId } from "./types";

const views: Array<{ id: ViewId; label: string; icon: typeof UserRoundCog }> = [
  { id: "agents", label: "Agents", icon: UserRoundCog },
  { id: "memories", label: "Memories", icon: Brain },
  { id: "tags", label: "Tags", icon: Tags },
  { id: "settings", label: "Settings", icon: Settings },
];

const viewTitles: Record<ViewId, { title: string; context: string }> = {
  agents: { title: "Agent access", context: "Global store" },
  memories: { title: "Memory explorer", context: "All stores" },
  tags: { title: "Tags", context: "6 indexed" },
  settings: { title: "Settings", context: "Local runtime" },
};

function roleLabel(role: AgentRole): string {
  if (role === "writer") return "Writer";
  if (role === "reader") return "Reader";
  return "No access";
}

function hookLabel(state: ManagedHookState): string {
  if (state === "active") return "Hooks active";
  if (state === "partial") return "Hooks partial";
  if (state === "invalid") return "Hooks invalid";
  if (state === "missing") return "Hooks off";
  return "Not applicable";
}

function configLabel(agent: AgentSummary): string {
  if (agent.configIssue) return "Config issue";
  if (agent.managed) return "Managed";
  if (agent.configured) return "Config found";
  return "Not configured";
}

function Brand() {
  return (
    <div className="brand" aria-label="Momonogi">
      <img className="brand__mark" src={markUrl} alt="" width="40" height="40" />
      <span className="brand__name">Momonogi</span>
    </div>
  );
}

function SideRail({ active, onChange }: { active: ViewId; onChange: (view: ViewId) => void }) {
  return (
    <aside className="side-rail">
      <Brand />
      <nav className="side-rail__nav" aria-label="Primary">
        {views.map(({ id, label, icon: Icon }) => (
          <button
            className="nav-button"
            data-active={active === id}
            key={id}
            type="button"
            aria-current={active === id ? "page" : undefined}
            aria-label={label}
            title={label}
            onClick={() => onChange(id)}
          >
            <Icon aria-hidden="true" size={19} strokeWidth={1.8} />
            <span>{label}</span>
          </button>
        ))}
      </nav>
      <div className="side-rail__store" aria-label="Store status">
        <span className="status-dot" aria-hidden="true" />
        <span>Store ready</span>
      </div>
    </aside>
  );
}

function Toolbar({
  active,
  query,
  setQuery,
  onRefresh,
  refreshing,
}: {
  active: ViewId;
  query: string;
  setQuery: (query: string) => void;
  onRefresh: () => void;
  refreshing: boolean;
}) {
  const current = viewTitles[active];
  return (
    <header className="toolbar">
      <div className="toolbar__title">
        <span>{current.context}</span>
        <h1>{current.title}</h1>
      </div>
      {active !== "settings" && (
        <label className="search-field">
          <Search aria-hidden="true" size={17} />
          <span className="sr-only">Search {active}</span>
          <input
            type="search"
            value={query}
            placeholder={`Search ${active}`}
            onChange={(event) => setQuery(event.target.value)}
          />
          {query && (
            <button type="button" aria-label="Clear search" onClick={() => setQuery("")}>
              <X aria-hidden="true" size={15} />
            </button>
          )}
        </label>
      )}
      <button
        className="icon-button toolbar__refresh"
        type="button"
        aria-label="Refresh"
        aria-busy={refreshing}
        title={refreshing ? "Refreshing" : "Refresh"}
        disabled={refreshing}
        onClick={onRefresh}
      >
        <RefreshCw aria-hidden="true" size={18} />
      </button>
    </header>
  );
}

function AgentInspector({ agent, onClose }: { agent: AgentSummary; onClose: () => void }) {
  return (
    <aside className="store-inspector agent-inspector" aria-labelledby="agent-inspector-heading">
      <div className="store-inspector__head">
        <AgentLogo agentId={agent.id} name={agent.name} command={agent.command} />
        <h3 id="agent-inspector-heading">{agent.name}</h3>
        <button className="icon-button inspector-close" type="button" aria-label="Close Agent details" title="Close" onClick={onClose}>
          <X aria-hidden="true" size={16} />
        </button>
      </div>
      <dl>
        <div><dt>Command</dt><dd><code>{agent.command || "-"}</code></dd></div>
        <div><dt>Installed</dt><dd>{agent.installed ? "Yes" : "No"}</dd></div>
        <div><dt>Access</dt><dd>{roleLabel(agent.role)}</dd></div>
        <div><dt>Rules</dt><dd>{configLabel(agent)}</dd></div>
        <div><dt>Hooks</dt><dd>{hookLabel(agent.hookState)}</dd></div>
      </dl>
      <div className="config-paths">
        <span>Configuration paths</span>
        {agent.configPaths.length > 0 ? agent.configPaths.map((path) => <code key={path} title={path}>{path}</code>) : <p>No host adapter paths</p>}
      </div>
      {agent.configIssue && <p className="config-issue"><CircleAlert aria-hidden="true" size={14} />{agent.configIssue}</p>}
    </aside>
  );
}

function StoreInspector({ discovery }: { discovery: AgentDiscoveryPayload | null }) {
  return (
    <aside className="store-inspector" aria-labelledby="active-store-heading">
      <div className="store-inspector__head">
        <Database aria-hidden="true" size={18} />
        <h3 id="active-store-heading">Active store</h3>
      </div>
      <dl>
        <div><dt>Scope</dt><dd>Global</dd></div>
        <div><dt>Health</dt><dd><span className="status-dot" data-state={discovery?.storeAvailable === false ? "warning" : undefined} />{discovery?.storeAvailable === false ? "Unavailable" : "Ready"}</dd></div>
        <div><dt>Schema</dt><dd><code>v1</code></dd></div>
        <div><dt>Revision</dt><dd><code>{discovery?.storeRevision ?? "-"}</code></dd></div>
        <div><dt>Root</dt><dd><code className="path-value" title={discovery?.storeRoot}>{discovery?.storeRoot ?? "loading"}</code></dd></div>
      </dl>
      {discovery?.storeIssue && <p className="config-issue"><CircleAlert aria-hidden="true" size={14} />{discovery.storeIssue}</p>}
      <button className="secondary-button" type="button">
        <FolderOpen aria-hidden="true" size={16} />
        Open folder
      </button>
    </aside>
  );
}

const roles: Array<{ value: AgentRole; label: string }> = [
  { value: "writer", label: "Writer" },
  { value: "reader", label: "Reader" },
  { value: "none", label: "None" },
];

function RoleSelector({
  agent,
  disabled,
  finalWriter,
  onChange,
}: {
  agent: AgentSummary;
  disabled: boolean;
  finalWriter: boolean;
  onChange: (role: AgentRole) => void;
}) {
  return (
    <div className="role-selector" role="group" aria-label={`${agent.name} access`}>
      {roles.map((role) => {
        const protectedRole = finalWriter && agent.role === "writer" && role.value !== "writer";
        return (
          <button
            key={role.value}
            type="button"
            aria-label={`${agent.name}: ${role.label}`}
            aria-pressed={agent.role === role.value}
            disabled={disabled || protectedRole}
            title={protectedRole ? "At least one writer is required" : `${role.label} access`}
            onClick={() => onChange(role.value)}
          >
            {role.label}
          </button>
        );
      })}
    </div>
  );
}

interface AccessNotice {
  state: "ok" | "error" | "warning";
  text: string;
}

function AgentsView({
  query,
  discovery,
  loading,
  error,
  accessNotice,
  savingAgentId,
  onSetAccess,
}: {
  query: string;
  discovery: AgentDiscoveryPayload | null;
  loading: boolean;
  error: string | null;
  accessNotice: AccessNotice | null;
  savingAgentId: string | null;
  onSetAccess: (agentId: string, role: AgentRole, actor: string) => void;
}) {
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [actor, setActor] = useState("");
  const allAgents = discovery?.agents ?? [];
  const agents = allAgents.filter((agent) =>
    `${agent.name} ${agent.id} ${agent.command}`.toLowerCase().includes(query.toLowerCase()),
  );
  const writers = allAgents.filter((agent) => agent.role === "writer").length;
  const readers = allAgents.filter((agent) => agent.role === "reader").length;
  const installed = allAgents.filter((agent) => agent.installed).length;
  const selectedAgent = allAgents.find((agent) => agent.id === selectedAgentId);

  useEffect(() => {
    if (actor && !allAgents.some((agent) => agent.id === actor && agent.role === "writer")) {
      setActor("");
    }
  }, [actor, allAgents]);

  return (
    <section className="view agents-view" aria-labelledby="agents-heading">
      <div className="section-heading">
        <div>
          <h2 id="agents-heading">Access matrix</h2>
          <p>Roles in the active store manifest</p>
        </div>
        <label className="actor-picker">
          <ShieldCheck aria-hidden="true" size={15} />
          <span>Writer identity</span>
          <select aria-label="Writer identity" value={actor} onChange={(event) => setActor(event.target.value)}>
            <option value="">Choose writer</option>
            {allAgents.filter((agent) => agent.role === "writer").map((agent) => (
              <option key={agent.id} value={agent.id}>{agent.name}</option>
            ))}
          </select>
        </label>
      </div>

      {(accessNotice || error) && (
        <div className="access-notice" data-state={accessNotice?.state ?? "error"} role={accessNotice?.state === "error" || error ? "alert" : "status"}>
          {accessNotice?.state === "ok" ? <Check aria-hidden="true" size={15} /> : <CircleAlert aria-hidden="true" size={15} />}
          <span>{accessNotice?.text ?? error}</span>
        </div>
      )}

      <div className="metric-strip" aria-label="Agent summary">
        <div><strong>{allAgents.length}</strong><span>Detected</span></div>
        <div><strong>{writers}</strong><span>Writers</span></div>
        <div><strong>{readers}</strong><span>Readers</span></div>
        <div><strong>{installed} / {allAgents.length}</strong><span>Installed</span></div>
      </div>

      <div className="workbench-grid">
        <div className="agent-list" role="list">
          {agents.map((agent) => (
            <article className="agent-row" key={agent.id} role="listitem">
              <div className="agent-row__identity">
                <AgentLogo agentId={agent.id} name={agent.name} command={agent.command} />
                <div>
                  <h3>{agent.name}</h3>
                  <code>{agent.command}</code>
                </div>
              </div>
              <div className="agent-row__signals">
                <span className="signal" data-state={agent.installed ? "ok" : "off"}>
                  {agent.installed ? <Check aria-hidden="true" size={13} /> : <X aria-hidden="true" size={13} />}
                  {agent.installed ? "Installed" : "Not installed"}
                </span>
                <span className="signal" data-state={agent.managed ? "ok" : agent.configured ? "quiet" : "off"} title={agent.configIssue}>
                  {agent.managed ? <ShieldCheck aria-hidden="true" size={13} /> : <CircleAlert aria-hidden="true" size={13} />}
                  {configLabel(agent)}
                </span>
                {agent.hookState !== "not-applicable" && (
                  <span className="signal" data-state={agent.hookState === "active" ? "ok" : "quiet"}>
                    {agent.hookState === "active" ? <Check aria-hidden="true" size={13} /> : <CircleAlert aria-hidden="true" size={13} />}
                    {hookLabel(agent.hookState)}
                  </span>
                )}
              </div>
              <RoleSelector
                agent={agent}
                disabled={!actor || !discovery?.storeEtag || Boolean(savingAgentId)}
                finalWriter={writers === 1}
                onChange={(role) => onSetAccess(agent.id, role, actor)}
              />
              <button className="icon-button" type="button" aria-label={`Open ${agent.name}`} title={`Open ${agent.name}`} onClick={() => setSelectedAgentId(agent.id)}>
                <ChevronRight aria-hidden="true" size={17} />
              </button>
            </article>
          ))}
          {agents.length === 0 && <EmptyState label={loading ? "Scanning local Agents" : error ? "Agent discovery failed" : "No Agents match this search"} />}
        </div>
        {selectedAgent ? <AgentInspector agent={selectedAgent} onClose={() => setSelectedAgentId(null)} /> : <StoreInspector discovery={discovery} />}
      </div>
    </section>
  );
}

function memoryKey(memory: MemorySummary): string {
  return `${memory.storePath}:${memory.archived ? "archive" : "active"}:${memory.slug}`;
}

function MemoriesView({
  query,
  index,
  loading,
  error,
}: {
  query: string;
  index: MemoryIndexPayload | null;
  loading: boolean;
  error: string | null;
}) {
  const [memoryType, setMemoryType] = useState("all");
  const [status, setStatus] = useState("all");
  const [scope, setScope] = useState("all");
  const [archive, setArchive] = useState("all");
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [detail, setDetail] = useState<MemoryDetailPayload | null>(null);
  const [detailError, setDetailError] = useState<string | null>(null);

  const memories = useMemo(() => {
    const search = query.trim().toLowerCase();
    return (index?.notes ?? []).filter((memory) => {
      if (memoryType !== "all" && memory.memoryType !== memoryType) return false;
      if (status !== "all" && memory.status !== status) return false;
      if (scope !== "all" && memory.scope !== scope) return false;
      if (archive === "active" && memory.archived) return false;
      if (archive === "archived" && !memory.archived) return false;
      if (!search) return true;
      return [memory.name, memory.description, memory.slug, memory.storeId, ...memory.tags]
        .some((value) => value.toLowerCase().includes(search));
    });
  }, [archive, index?.notes, memoryType, query, scope, status]);

  const selected = memories.find((memory) => memoryKey(memory) === selectedKey) ?? null;
  const globalMemories = memories.filter((memory) => memory.storeKind === "global");
  const projectMemories = memories.filter((memory) => memory.storeKind === "project");

  useEffect(() => {
    if (memories.length === 0) {
      setSelectedKey(null);
      setDetail(null);
    } else if (!selected) {
      setSelectedKey(memoryKey(memories[0]));
    }
  }, [memories, selected]);

  useEffect(() => {
    if (!selected) return;
    let cancelled = false;
    setDetail(null);
    setDetailError(null);
    void getMemoryDetail(selected.storePath, selected.slug, selected.archived)
      .then((value) => {
        if (!cancelled) setDetail(value);
      })
      .catch((cause) => {
        if (!cancelled) setDetailError(cause instanceof Error ? cause.message : String(cause));
      });
    return () => {
      cancelled = true;
    };
  }, [selected]);

  function rows(items: MemorySummary[]) {
    return items.map((memory) => (
      <button
        className="memory-row"
        data-active={selectedKey === memoryKey(memory)}
        key={memoryKey(memory)}
        type="button"
        role="option"
        aria-selected={selectedKey === memoryKey(memory)}
        onClick={() => setSelectedKey(memoryKey(memory))}
      >
        <span className="memory-row__top"><strong>{memory.name}</strong><time>{memory.updated}</time></span>
        <span className="memory-row__excerpt">{memory.description}</span>
        <span className="memory-row__meta">
          <span>{memory.storeId}</span>
          <span>{memory.memoryType}</span>
          <span>{memory.archived ? "archived" : memory.scope}</span>
        </span>
      </button>
    ));
  }

  return (
    <section className="view memories-view" aria-labelledby="memories-heading">
      <div className="section-heading">
        <div>
          <h2 id="memories-heading">Indexed notes</h2>
          <p>Global and registered project stores</p>
        </div>
        <span className="count-label">{memories.length} notes</span>
      </div>
      <div className="memory-filters" aria-label="Memory filters">
        <label>Type<select aria-label="Memory type" value={memoryType} onChange={(event) => setMemoryType(event.target.value)}><option value="all">All</option><option value="user">User</option><option value="feedback">Feedback</option><option value="project">Project</option><option value="reference">Reference</option></select></label>
        <label>Status<select aria-label="Memory status" value={status} onChange={(event) => setStatus(event.target.value)}><option value="all">All</option><option value="active">Active</option><option value="archived">Archived</option></select></label>
        <label>Scope<select aria-label="Memory scope" value={scope} onChange={(event) => setScope(event.target.value)}><option value="all">All</option><option value="global">Global</option><option value="repo">Repo</option></select></label>
        <label>Archive<select aria-label="Archive state" value={archive} onChange={(event) => setArchive(event.target.value)}><option value="all">All</option><option value="active">Active only</option><option value="archived">Archived only</option></select></label>
      </div>
      {(error || (index?.issues.length ?? 0) > 0) && (
        <div className="access-notice memory-issue" role="alert" title={index?.issues.map((issue) => `${issue.slug ?? issue.storePath}: ${issue.message}`).join("\n")}>
          <CircleAlert aria-hidden="true" size={15} />
          {error ?? `${index?.issues.length} memory item${index?.issues.length === 1 ? "" : "s"} could not be read.`}
        </div>
      )}
      <div className="memory-workbench">
        <div className="memory-list" role="listbox" aria-label="Memories">
          {globalMemories.length > 0 && <div className="memory-group" role="group" aria-label="Global"><h3>Global</h3>{rows(globalMemories)}</div>}
          {projectMemories.length > 0 && <div className="memory-group" role="group" aria-label="Projects"><h3>Projects</h3>{rows(projectMemories)}</div>}
          {memories.length === 0 && <EmptyState label={loading ? "Reading registered stores" : "No memories match these filters"} />}
        </div>
        <article className="memory-detail" aria-live="polite">
          {detail ? (
            <>
              <div className="memory-detail__head">
                <div>
                  <span>{detail.summary.storeKind} / {detail.summary.storeId}</span>
                  <h3>{detail.summary.name}</h3>
                </div>
                <code>{detail.summary.slug}</code>
              </div>
              <div className="memory-detail__meta">
                <span>{detail.summary.memoryType}</span>
                <span>{detail.summary.status}</span>
                <span>{detail.summary.scope}</span>
                <span>r{detail.summary.revision}</span>
              </div>
              <p>{detail.summary.description}</p>
              <pre className="memory-body">{detail.body}</pre>
              <div className="tag-line" aria-label="Tags">
                {detail.summary.tags.map((tag) => <span key={tag}>{tag}</span>)}
              </div>
            </>
          ) : detailError ? (
            <div className="memory-detail-error" role="alert"><CircleAlert aria-hidden="true" size={18} /><p>{detailError}</p></div>
          ) : (
            <EmptyState label={selected ? "Reading complete memory" : "Select a memory to inspect"} />
          )}
        </article>
      </div>
    </section>
  );
}

function TagsView({ query }: { query: string }) {
  const tags = mockTags.filter((tag) => tag.name.toLowerCase().includes(query.toLowerCase()));
  return (
    <section className="view tags-view" aria-labelledby="tags-heading">
      <div className="section-heading">
        <div><h2 id="tags-heading">Tag index</h2><p>Normalized across registered stores</p></div>
        <button className="text-button" type="button">Manage tags<ChevronRight aria-hidden="true" size={16} /></button>
      </div>
      <div className="tag-table" role="table" aria-label="Tags">
        <div className="tag-table__head" role="row">
          <span role="columnheader">Tag</span><span role="columnheader">Scope</span><span role="columnheader">Notes</span><span />
        </div>
        {tags.map((tag) => (
          <div className="tag-table__row" role="row" key={tag.name}>
            <strong role="cell">{tag.name}</strong>
            <span role="cell">{tag.scope}</span>
            <span role="cell" className="mono-number">{tag.count}</span>
            <button className="icon-button" type="button" aria-label={`Open ${tag.name}`} title={`Open ${tag.name}`}>
              <ChevronRight aria-hidden="true" size={17} />
            </button>
          </div>
        ))}
        {tags.length === 0 && <EmptyState label="No tags match this search" />}
      </div>
    </section>
  );
}

function SettingsView({
  bootstrap,
  stores,
  registryError,
  registryBusy,
  onRegister,
  onRemove,
}: {
  bootstrap: BootstrapPayload | null;
  stores: StoreSummary[];
  registryError: string | null;
  registryBusy: boolean;
  onRegister: (path: string) => Promise<boolean>;
  onRemove: (path: string) => Promise<void>;
}) {
  const [projectPath, setProjectPath] = useState("");

  async function submitProject(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (await onRegister(projectPath)) setProjectPath("");
  }

  return (
    <section className="view settings-view" aria-labelledby="settings-heading">
      <div className="section-heading">
        <div><h2 id="settings-heading">Runtime</h2><p>Desktop bridge and Momonogi core</p></div>
      </div>
      <dl className="settings-list">
        <div><dt>Application</dt><dd>Momonogi Desktop</dd></div>
        <div><dt>Version</dt><dd><code>{bootstrap?.appVersion ?? "loading"}</code></dd></div>
        <div><dt>Core schema</dt><dd><code>v{bootstrap?.coreSchema ?? "-"}</code></dd></div>
        <div><dt>Bridge</dt><dd><span className="runtime-badge">{bootstrap?.bridge ?? "loading"}</span></dd></div>
      </dl>

      <div className="section-heading store-registry-heading">
        <div><h2>Store registry</h2><p>Global and explicitly registered project stores</p></div>
        <span className="count-label">{stores.length} stores</span>
      </div>
      <form className="store-register" onSubmit={(event) => void submitProject(event)}>
        <label>
          <span className="sr-only">Project store path</span>
          <FolderPlus aria-hidden="true" size={17} />
          <input
            value={projectPath}
            aria-label="Project store path"
            placeholder="/path/to/project/.momonogi"
            onChange={(event) => setProjectPath(event.target.value)}
          />
        </label>
        <button className="secondary-button" type="submit" disabled={registryBusy || !projectPath.trim()}>
          Register
        </button>
      </form>
      {registryError && <div className="access-notice registry-notice" role="alert"><CircleAlert aria-hidden="true" size={15} />{registryError}</div>}
      <div className="store-list" role="list" aria-label="Registered stores">
        {stores.map((store) => (
          <article className="store-row" role="listitem" key={`${store.kind}:${store.path}`}>
            <Database aria-hidden="true" size={18} />
            <div className="store-row__identity">
              <h3>{store.storeId ?? (store.kind === "global" ? "Global store" : "Project store")}</h3>
              <code title={store.path}>{store.path}</code>
            </div>
            <span className="store-health" data-health={store.health}>{store.health}</span>
            <code className="store-revision">r{store.revision ?? "-"}</code>
            {store.kind === "project" ? (
              <button
                className="icon-button"
                type="button"
                aria-label={`Remove ${store.storeId ?? store.path}`}
                title="Remove from registry"
                disabled={registryBusy}
                onClick={() => void onRemove(store.path)}
              >
                <Trash2 aria-hidden="true" size={16} />
              </button>
            ) : <span className="store-row__fixed" />}
            {store.issue && <p className="store-row__issue">{store.issue}</p>}
          </article>
        ))}
      </div>
    </section>
  );
}

function EmptyState({ label }: { label: string }) {
  return (
    <div className="empty-state">
      <Search aria-hidden="true" size={20} />
      <p>{label}</p>
    </div>
  );
}

export function App() {
  const [active, setActive] = useState<ViewId>("agents");
  const [query, setQuery] = useState("");
  const [bootstrap, setBootstrap] = useState<BootstrapPayload | null>(null);
  const [agentDiscovery, setAgentDiscovery] = useState<AgentDiscoveryPayload | null>(null);
  const [discoveryError, setDiscoveryError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [savingAgentId, setSavingAgentId] = useState<string | null>(null);
  const [accessNotice, setAccessNotice] = useState<AccessNotice | null>(null);
  const [stores, setStores] = useState<StoreSummary[]>([]);
  const [registryError, setRegistryError] = useState<string | null>(null);
  const [registryBusy, setRegistryBusy] = useState(false);
  const [memoryIndex, setMemoryIndex] = useState<MemoryIndexPayload | null>(null);
  const [memoryError, setMemoryError] = useState<string | null>(null);

  const refreshData = useCallback(async () => {
    setRefreshing(true);
    setDiscoveryError(null);
    setRegistryError(null);
    setMemoryError(null);
    const [bootstrapResult, discoveryResult, registryResult, memoryResult] = await Promise.allSettled([
      getBootstrap(),
      getAgentDiscovery(),
      getStoreRegistry(),
      getMemoryIndex(),
    ]);
    if (bootstrapResult.status === "fulfilled") setBootstrap(bootstrapResult.value);
    if (discoveryResult.status === "fulfilled") {
      setAgentDiscovery(discoveryResult.value);
    } else {
      setDiscoveryError(discoveryResult.reason instanceof Error ? discoveryResult.reason.message : String(discoveryResult.reason));
    }
    if (registryResult.status === "fulfilled") {
      setStores(registryResult.value);
    } else {
      setRegistryError(registryResult.reason instanceof Error ? registryResult.reason.message : String(registryResult.reason));
    }
    if (memoryResult.status === "fulfilled") {
      setMemoryIndex(memoryResult.value);
    } else {
      setMemoryError(memoryResult.reason instanceof Error ? memoryResult.reason.message : String(memoryResult.reason));
    }
    setRefreshing(false);
  }, []);

  useEffect(() => {
    void refreshData();
  }, [refreshData]);

  const updateAccess = useCallback(async (agentId: string, role: AgentRole, actor: string) => {
    const ifMatch = agentDiscovery?.storeEtag;
    if (!ifMatch) {
      setAccessNotice({ state: "error", text: "The active store has no writable manifest." });
      return;
    }
    setSavingAgentId(agentId);
    setAccessNotice(null);
    try {
      const result = await setAgentAccess({ agentId, role, actor, ifMatch });
      await refreshData();
      setAccessNotice({
        state: "ok",
        text: result.changed ? `Access updated at revision ${result.revision}.` : "Access was already current.",
      });
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      if (message.includes("etag conflict")) {
        await refreshData();
        setAccessNotice({ state: "warning", text: "The store changed elsewhere. Current roles were reloaded." });
      } else {
        setAccessNotice({ state: "error", text: message });
      }
    } finally {
      setSavingAgentId(null);
    }
  }, [agentDiscovery?.storeEtag, refreshData]);

  const addProjectStore = useCallback(async (path: string) => {
    setRegistryBusy(true);
    setRegistryError(null);
    try {
      setStores(await registerProjectStore(path));
      setMemoryIndex(await getMemoryIndex());
      return true;
    } catch (cause) {
      setRegistryError(cause instanceof Error ? cause.message : String(cause));
      return false;
    } finally {
      setRegistryBusy(false);
    }
  }, []);

  const removeStore = useCallback(async (path: string) => {
    setRegistryBusy(true);
    setRegistryError(null);
    try {
      setStores(await removeProjectStore(path));
      setMemoryIndex(await getMemoryIndex());
    } catch (cause) {
      setRegistryError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setRegistryBusy(false);
    }
  }, []);

  const content = useMemo(() => {
    if (active === "agents") return (
      <AgentsView
        query={query}
        discovery={agentDiscovery}
        loading={refreshing && !agentDiscovery}
        error={discoveryError}
        accessNotice={accessNotice}
        savingAgentId={savingAgentId}
        onSetAccess={(agentId, role, actor) => void updateAccess(agentId, role, actor)}
      />
    );
    if (active === "memories") return <MemoriesView query={query} index={memoryIndex} loading={refreshing && !memoryIndex} error={memoryError} />;
    if (active === "tags") return <TagsView query={query} />;
    return (
      <SettingsView
        bootstrap={bootstrap}
        stores={stores}
        registryError={registryError}
        registryBusy={registryBusy}
        onRegister={addProjectStore}
        onRemove={removeStore}
      />
    );
  }, [accessNotice, active, addProjectStore, agentDiscovery, bootstrap, discoveryError, memoryError, memoryIndex, query, refreshing, registryBusy, registryError, removeStore, savingAgentId, stores, updateAccess]);

  function changeView(view: ViewId) {
    setActive(view);
    setQuery("");
  }

  return (
    <div className="app-shell">
      <SideRail active={active} onChange={changeView} />
      <div className="app-shell__main">
        <Toolbar active={active} query={query} setQuery={setQuery} onRefresh={() => void refreshData()} refreshing={refreshing} />
        <main>{content}</main>
        <footer className="status-line">
          <span><span className="status-dot" aria-hidden="true" />{bootstrap?.bridge === "desktop" ? "Desktop bridge" : "Development bridge"}</span>
          <span>Schema v{bootstrap?.coreSchema ?? "-"}</span>
          <span className="status-line__version">Momonogi {bootstrap?.appVersion ?? "loading"}</span>
        </footer>
      </div>
    </div>
  );
}
