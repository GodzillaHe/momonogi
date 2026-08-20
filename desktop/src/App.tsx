import {
  Brain,
  Check,
  ChevronRight,
  CircleAlert,
  Database,
  FolderOpen,
  RefreshCw,
  Search,
  Settings,
  ShieldCheck,
  Tags,
  UserRoundCog,
  X,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { AgentLogo } from "./AgentLogo";
import { getAgentDiscovery, getBootstrap } from "./bridge";
import markUrl from "./assets/momonogi-mark.svg";
import { mockMemories, mockTags } from "./mock-data";
import type { AgentDiscoveryPayload, AgentRole, AgentSummary, BootstrapPayload, ManagedHookState, ViewId } from "./types";

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

function AgentsView({ query, discovery, loading, error }: { query: string; discovery: AgentDiscoveryPayload | null; loading: boolean; error: string | null }) {
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const allAgents = discovery?.agents ?? [];
  const agents = allAgents.filter((agent) =>
    `${agent.name} ${agent.id} ${agent.command}`.toLowerCase().includes(query.toLowerCase()),
  );
  const writers = allAgents.filter((agent) => agent.role === "writer").length;
  const readers = allAgents.filter((agent) => agent.role === "reader").length;
  const installed = allAgents.filter((agent) => agent.installed).length;
  const selectedAgent = allAgents.find((agent) => agent.id === selectedAgentId);

  return (
    <section className="view agents-view" aria-labelledby="agents-heading">
      <div className="section-heading">
        <div>
          <h2 id="agents-heading">Access matrix</h2>
          <p>Roles in the active store manifest</p>
        </div>
        <button className="text-button" type="button">
          Configure
          <ChevronRight aria-hidden="true" size={16} />
        </button>
      </div>

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
              <span className="role-badge" data-role={agent.role}>{roleLabel(agent.role)}</span>
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

function MemoriesView({ query }: { query: string }) {
  const memories = mockMemories.filter((memory) =>
    `${memory.title} ${memory.excerpt} ${memory.tags.join(" ")}`.toLowerCase().includes(query.toLowerCase()),
  );
  const [selectedSlug, setSelectedSlug] = useState(mockMemories[0].slug);
  const selected = memories.find((memory) => memory.slug === selectedSlug) ?? memories[0];

  return (
    <section className="view memories-view" aria-labelledby="memories-heading">
      <div className="section-heading">
        <div>
          <h2 id="memories-heading">Indexed notes</h2>
          <p>Global and registered project stores</p>
        </div>
        <span className="count-label">{memories.length} notes</span>
      </div>
      <div className="memory-workbench">
        <div className="memory-list" role="listbox" aria-label="Memories">
          {memories.map((memory) => (
            <button
              className="memory-row"
              data-active={selected?.slug === memory.slug}
              key={memory.slug}
              type="button"
              role="option"
              aria-selected={selected?.slug === memory.slug}
              onClick={() => setSelectedSlug(memory.slug)}
            >
              <span className="memory-row__top"><strong>{memory.title}</strong><time>{memory.updated}</time></span>
              <span className="memory-row__excerpt">{memory.excerpt}</span>
              <span className="memory-row__meta"><span>{memory.scope}</span><span>{memory.type}</span></span>
            </button>
          ))}
          {memories.length === 0 && <EmptyState label="No memories match this search" />}
        </div>
        <article className="memory-detail" aria-live="polite">
          {selected ? (
            <>
              <div className="memory-detail__head">
                <div>
                  <span>{selected.scope}{selected.project ? ` / ${selected.project}` : ""}</span>
                  <h3>{selected.title}</h3>
                </div>
                <code>{selected.slug}</code>
              </div>
              <p>{selected.excerpt}</p>
              <div className="tag-line" aria-label="Tags">
                {selected.tags.map((tag) => <span key={tag}>{tag}</span>)}
              </div>
            </>
          ) : (
            <EmptyState label="Select a memory to inspect" />
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

function SettingsView({ bootstrap }: { bootstrap: BootstrapPayload | null }) {
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

  const refreshData = useCallback(async () => {
    setRefreshing(true);
    setDiscoveryError(null);
    const [bootstrapResult, discoveryResult] = await Promise.allSettled([getBootstrap(), getAgentDiscovery()]);
    if (bootstrapResult.status === "fulfilled") setBootstrap(bootstrapResult.value);
    if (discoveryResult.status === "fulfilled") {
      setAgentDiscovery(discoveryResult.value);
    } else {
      setDiscoveryError(discoveryResult.reason instanceof Error ? discoveryResult.reason.message : String(discoveryResult.reason));
    }
    setRefreshing(false);
  }, []);

  useEffect(() => {
    void refreshData();
  }, [refreshData]);

  const content = useMemo(() => {
    if (active === "agents") return <AgentsView query={query} discovery={agentDiscovery} loading={refreshing && !agentDiscovery} error={discoveryError} />;
    if (active === "memories") return <MemoriesView query={query} />;
    if (active === "tags") return <TagsView query={query} />;
    return <SettingsView bootstrap={bootstrap} />;
  }, [active, agentDiscovery, bootstrap, discoveryError, query, refreshing]);

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
