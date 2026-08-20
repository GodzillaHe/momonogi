import {
  Brain,
  Check,
  ChevronRight,
  CircleAlert,
  Database,
  FileCog,
  FolderPlus,
  FolderOpen,
  Plus,
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
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { AgentLogo } from "./AgentLogo";
import {
  getAgentDiscovery,
  getBootstrap,
  getMemoryDetail,
  getMemoryIndex,
  getStoreRegistry,
  openStoreFolder,
  changeMemoryTag,
  applyAgentConfiguration,
  previewAgentConfiguration,
  registerProjectStore,
  removeProjectStore,
  setAgentAccess,
} from "./bridge";
import markUrl from "./assets/momonogi-mark.svg";
import { setAppLanguage, type AppLanguage } from "./i18n";
import type { AgentDiscoveryPayload, AgentRole, AgentSummary, BootstrapPayload, ConfigurationAction, ConfigurationPlanPayload, ManagedHookState, MemoryDetailPayload, MemoryIndexPayload, MemorySummary, StoreSummary, ViewId } from "./types";

const views: Array<{ id: ViewId; icon: typeof UserRoundCog }> = [
  { id: "agents", icon: UserRoundCog },
  { id: "memories", icon: Brain },
  { id: "tags", icon: Tags },
  { id: "settings", icon: Settings },
];

function roleLabel(t: TFunction, role: AgentRole, short = false): string {
  return t(role === "none" && short ? "role.noneShort" : `role.${role}`);
}

function hookLabel(t: TFunction, state: ManagedHookState): string {
  return t(`hooks.${state === "not-applicable" ? "notApplicable" : state}`);
}

function configLabel(t: TFunction, agent: AgentSummary): string {
  if (agent.configIssue) return t("config.issue");
  if (agent.managed) return t("config.managed");
  if (agent.configured) return t("config.found");
  return t("config.notConfigured");
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
  const { t } = useTranslation();
  return (
    <aside className="side-rail">
      <Brand />
      <nav className="side-rail__nav" aria-label={t("nav.primary")}>
        {views.map(({ id, icon: Icon }) => {
          const label = t(`nav.${id}`);
          return (
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
          );
        })}
      </nav>
      <div className="side-rail__store" aria-label={t("nav.storeStatus")}>
        <span className="status-dot" aria-hidden="true" />
        <span>{t("nav.storeReady")}</span>
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
  const { t, i18n } = useTranslation();
  const language = i18n.resolvedLanguage === "zh-CN" ? "zh-CN" : "en";
  const searchLabel = t(`toolbar.${active}.search`);
  return (
    <header className="toolbar">
      <div className="toolbar__title">
        <span>{t(`toolbar.${active}.context`)}</span>
        <h1>{t(`toolbar.${active}.title`)}</h1>
      </div>
      {active !== "settings" && (
        <label className="search-field">
          <Search aria-hidden="true" size={17} />
          <span className="sr-only">{searchLabel}</span>
          <input
            type="search"
            value={query}
            placeholder={searchLabel}
            onChange={(event) => setQuery(event.target.value)}
          />
          {query && (
            <button type="button" aria-label={t("toolbar.clearSearch")} onClick={() => setQuery("")}>
              <X aria-hidden="true" size={15} />
            </button>
          )}
        </label>
      )}
      <div className="toolbar__actions">
        <div className="language-switcher" role="group" aria-label={t("language.group")}>
          {(["zh-CN", "en"] as AppLanguage[]).map((value) => (
            <button
              key={value}
              type="button"
              aria-label={t(value === "zh-CN" ? "language.chinese" : "language.english")}
              aria-pressed={language === value}
              onClick={() => void setAppLanguage(value)}
            >
              {value === "zh-CN" ? "中" : "EN"}
            </button>
          ))}
        </div>
        <button
          className="icon-button toolbar__refresh"
          type="button"
          aria-label={t("common.refresh")}
          aria-busy={refreshing}
          data-state={refreshing ? "loading" : undefined}
          title={t(refreshing ? "common.refreshing" : "common.refresh")}
          disabled={refreshing}
          onClick={onRefresh}
        >
          <RefreshCw aria-hidden="true" size={18} />
        </button>
      </div>
    </header>
  );
}

function AgentInspector({
  agent,
  busy,
  onClose,
  onPreviewConfiguration,
}: {
  agent: AgentSummary;
  busy: boolean;
  onClose: () => void;
  onPreviewConfiguration: () => void;
}) {
  const { t } = useTranslation();
  return (
    <aside className="store-inspector agent-inspector" aria-labelledby="agent-inspector-heading">
      <div className="store-inspector__head">
        <AgentLogo agentId={agent.id} name={agent.name} command={agent.command} />
        <h3 id="agent-inspector-heading">{agent.name}</h3>
        <button className="icon-button inspector-close" type="button" aria-label={t("agents.closeDetails")} title={t("common.close")} onClick={onClose}>
          <X aria-hidden="true" size={16} />
        </button>
      </div>
      <dl>
        <div><dt>{t("agents.command")}</dt><dd><code>{agent.command || "-"}</code></dd></div>
        <div><dt>{t("agents.installed")}</dt><dd>{t(agent.installed ? "common.yes" : "common.no")}</dd></div>
        <div><dt>{t("agents.access")}</dt><dd>{roleLabel(t, agent.role)}</dd></div>
        <div><dt>{t("agents.rules")}</dt><dd>{configLabel(t, agent)}</dd></div>
        <div><dt>{t("agents.hooks")}</dt><dd>{hookLabel(t, agent.hookState)}</dd></div>
      </dl>
      <div className="config-paths">
        <span>{t("config.paths")}</span>
        {agent.configPaths.length > 0 ? agent.configPaths.map((path) => <code key={path} title={path}>{path}</code>) : <p>{t("config.noPaths")}</p>}
      </div>
      {agent.configIssue && <p className="config-issue"><CircleAlert aria-hidden="true" size={14} />{agent.configIssue}</p>}
      <button className="secondary-button" type="button" aria-busy={busy} data-state={busy ? "loading" : undefined} disabled={busy} onClick={onPreviewConfiguration}>
        {busy ? <RefreshCw aria-hidden="true" size={16} /> : <FileCog aria-hidden="true" size={16} />}
        {t("config.preview")}
      </button>
    </aside>
  );
}

function StoreInspector({
  discovery,
  onOpenFolder,
}: {
  discovery: AgentDiscoveryPayload | null;
  onOpenFolder: (path: string) => void;
}) {
  const { t } = useTranslation();
  return (
    <aside className="store-inspector" aria-labelledby="active-store-heading">
      <div className="store-inspector__head">
        <Database aria-hidden="true" size={18} />
        <h3 id="active-store-heading">{t("store.active")}</h3>
      </div>
      <dl>
        <div><dt>{t("store.scope")}</dt><dd>{t("store.global")}</dd></div>
        <div><dt>{t("store.health")}</dt><dd><span className="status-dot" data-state={discovery?.storeAvailable === false ? "warning" : undefined} />{t(discovery?.storeAvailable === false ? "store.unavailable" : "store.ready")}</dd></div>
        <div><dt>{t("store.schema")}</dt><dd><code>v1</code></dd></div>
        <div><dt>{t("store.revision")}</dt><dd><code>{discovery?.storeRevision ?? "-"}</code></dd></div>
        <div><dt>{t("store.root")}</dt><dd><code className="path-value" title={discovery?.storeRoot}>{discovery?.storeRoot ?? t("common.loading")}</code></dd></div>
      </dl>
      {discovery?.storeIssue && <p className="config-issue"><CircleAlert aria-hidden="true" size={14} />{discovery.storeIssue}</p>}
      <button className="secondary-button" type="button" disabled={!discovery?.storeAvailable} onClick={() => discovery && onOpenFolder(discovery.storeRoot)}>
        <FolderOpen aria-hidden="true" size={16} />
        {t("store.openFolder")}
      </button>
    </aside>
  );
}

const roles: AgentRole[] = ["writer", "reader", "none"];

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
  const { t } = useTranslation();
  return (
    <div className="role-selector" role="group" aria-label={t("role.group", { name: agent.name })}>
      {roles.map((role) => {
        const protectedRole = finalWriter && agent.role === "writer" && role !== "writer";
        const label = roleLabel(t, role, true);
        return (
          <button
            key={role}
            type="button"
            aria-label={t("role.option", { name: agent.name, role: label })}
            aria-pressed={agent.role === role}
            disabled={disabled || protectedRole}
            title={protectedRole ? t("role.finalWriter") : t("role.access", { role: label })}
            onClick={() => onChange(role)}
          >
            {label}
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

function configurationActionLabel(t: TFunction, action: ConfigurationAction): string {
  return t(`config.action.${action === "remove-managed" ? "removeManaged" : action}`);
}

function ConfigurationPreview({
  plan,
  agentName,
  busy,
  onApply,
  onDismiss,
}: {
  plan: ConfigurationPlanPayload;
  agentName: string;
  busy: boolean;
  onApply: () => void;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  const changed = plan.files.filter((file) => file.action !== "unchanged").length;
  return (
    <section className="configuration-preview" aria-labelledby="configuration-preview-heading">
      <div className="configuration-preview__head">
        <div>
          <span>{t("config.title", { role: roleLabel(t, plan.role ?? "none") })}</span>
          <h3 id="configuration-preview-heading">{agentName}</h3>
        </div>
        <button className="icon-button" type="button" aria-label={t("config.dismiss")} title={t("common.dismiss")} disabled={busy} onClick={onDismiss}>
          <X aria-hidden="true" size={16} />
        </button>
      </div>
      {plan.warnings.map((warning) => (
        <p className="configuration-preview__warning" key={warning}><CircleAlert aria-hidden="true" size={14} />{warning}</p>
      ))}
      <div className="configuration-file-list" role="list" aria-label={t("config.files")}>
        {plan.files.map((file) => (
          <div
            className="configuration-file"
            role="listitem"
            aria-label={`${file.path}, ${t(`config.kind.${file.kind}`)}, ${configurationActionLabel(t, file.action)}`}
            key={`${file.kind}:${file.path}`}
          >
            <FileCog aria-hidden="true" size={16} />
            <code title={file.path}>{file.path}</code>
            <span>{t(`config.kind.${file.kind}`)}</span>
            <strong data-action={file.action}>{configurationActionLabel(t, file.action)}</strong>
          </div>
        ))}
        {plan.files.length === 0 && <p className="configuration-preview__empty">{t("config.noHostPaths")}</p>}
      </div>
      <div className="configuration-preview__actions">
        <span>{changed === 0 ? t("config.current") : t("config.filesChange", { count: changed })}</span>
        <button className="sync-button" type="button" aria-busy={busy} data-state={busy ? "loading" : undefined} disabled={busy || changed === 0} onClick={onApply}>
          {busy ? <RefreshCw aria-hidden="true" size={16} /> : <Check aria-hidden="true" size={16} />}
          {t("config.apply")}
        </button>
      </div>
    </section>
  );
}

function AgentsView({
  query,
  discovery,
  loading,
  error,
  accessNotice,
  configurationNotice,
  configurationPlan,
  configurationBusy,
  savingAgentId,
  onSetAccess,
  onPreviewConfiguration,
  onApplyConfiguration,
  onDismissConfiguration,
  onOpenStoreFolder,
}: {
  query: string;
  discovery: AgentDiscoveryPayload | null;
  loading: boolean;
  error: string | null;
  accessNotice: AccessNotice | null;
  configurationNotice: AccessNotice | null;
  configurationPlan: ConfigurationPlanPayload | null;
  configurationBusy: boolean;
  savingAgentId: string | null;
  onSetAccess: (agentId: string, role: AgentRole, actor: string) => void;
  onPreviewConfiguration: (agentId: string) => void;
  onApplyConfiguration: () => void;
  onDismissConfiguration: () => void;
  onOpenStoreFolder: (path: string) => void;
}) {
  const { t } = useTranslation();
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
          <h2 id="agents-heading">{t("agents.heading")}</h2>
          <p>{t("agents.description")}</p>
        </div>
        <label className="actor-picker">
          <ShieldCheck aria-hidden="true" size={15} />
          <span>{t("agents.writerIdentity")}</span>
          <select aria-label={t("agents.writerIdentity")} value={actor} onChange={(event) => setActor(event.target.value)}>
            <option value="">{t("agents.chooseWriter")}</option>
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
      {configurationNotice && (
        <div className="access-notice configuration-notice" data-state={configurationNotice.state} role={configurationNotice.state === "error" ? "alert" : "status"}>
          {configurationNotice.state === "ok" ? <Check aria-hidden="true" size={15} /> : <CircleAlert aria-hidden="true" size={15} />}
          <span>{configurationNotice.text}</span>
        </div>
      )}

      <div className="metric-strip" aria-label={t("agents.summary")}>
        <div><strong>{allAgents.length}</strong><span>{t("agents.detected")}</span></div>
        <div><strong>{writers}</strong><span>{t("agents.writers")}</span></div>
        <div><strong>{readers}</strong><span>{t("agents.readers")}</span></div>
        <div><strong>{installed} / {allAgents.length}</strong><span>{t("agents.installed")}</span></div>
      </div>

      {configurationPlan && (
        <ConfigurationPreview
          plan={configurationPlan}
          agentName={allAgents.find((agent) => agent.id === configurationPlan.agentId)?.name ?? configurationPlan.agentId}
          busy={configurationBusy}
          onApply={onApplyConfiguration}
          onDismiss={onDismissConfiguration}
        />
      )}

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
                  {t(agent.installed ? "agents.installed" : "agents.notInstalled")}
                </span>
                <span className="signal" data-state={agent.managed ? "ok" : agent.configured ? "quiet" : "off"} title={agent.configIssue}>
                  {agent.managed ? <ShieldCheck aria-hidden="true" size={13} /> : <CircleAlert aria-hidden="true" size={13} />}
                  {configLabel(t, agent)}
                </span>
                {agent.hookState !== "not-applicable" && (
                  <span className="signal" data-state={agent.hookState === "active" ? "ok" : "quiet"}>
                    {agent.hookState === "active" ? <Check aria-hidden="true" size={13} /> : <CircleAlert aria-hidden="true" size={13} />}
                    {hookLabel(t, agent.hookState)}
                  </span>
                )}
              </div>
              <RoleSelector
                agent={agent}
                disabled={!actor || !discovery?.storeEtag || Boolean(savingAgentId)}
                finalWriter={writers === 1}
                onChange={(role) => onSetAccess(agent.id, role, actor)}
              />
              <button className="icon-button" type="button" aria-label={t("agents.open", { name: agent.name })} title={t("agents.open", { name: agent.name })} onClick={() => setSelectedAgentId(agent.id)}>
                <ChevronRight aria-hidden="true" size={17} />
              </button>
            </article>
          ))}
          {agents.length === 0 && <EmptyState label={t(loading ? "agents.scanning" : error ? "agents.discoveryFailed" : "agents.noMatch")} />}
        </div>
        {selectedAgent ? (
          <AgentInspector
            agent={selectedAgent}
            busy={configurationBusy}
            onClose={() => setSelectedAgentId(null)}
            onPreviewConfiguration={() => onPreviewConfiguration(selectedAgent.id)}
          />
        ) : <StoreInspector discovery={discovery} onOpenFolder={onOpenStoreFolder} />}
      </div>
    </section>
  );
}

function memoryKey(memory: MemorySummary): string {
  return `${memory.storePath}:${memory.archived ? "archive" : "active"}:${memory.slug}`;
}

function mockMemoryKey(memory: MemorySummary): string | null {
  if (!memory.etag.startsWith("mock-")) return null;
  return memory.slug.replace(/\.md$/, "");
}

function memoryText(t: TFunction, memory: MemorySummary, field: "name" | "description"): string {
  const key = mockMemoryKey(memory);
  return key ? t(`mock.${key}.${field}`, { defaultValue: memory[field] }) : memory[field];
}

function MemoriesView({
  query,
  index,
  loading,
  error,
  onRefreshIndex,
}: {
  query: string;
  index: MemoryIndexPayload | null;
  loading: boolean;
  error: string | null;
  onRefreshIndex: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [memoryType, setMemoryType] = useState("all");
  const [status, setStatus] = useState("all");
  const [scope, setScope] = useState("all");
  const [archive, setArchive] = useState("all");
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [detail, setDetail] = useState<MemoryDetailPayload | null>(null);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [tagActor, setTagActor] = useState("");
  const [tagInput, setTagInput] = useState("");
  const [tagBusy, setTagBusy] = useState(false);
  const [tagNotice, setTagNotice] = useState<AccessNotice | null>(null);

  const memories = useMemo(() => {
    const search = query.trim().toLowerCase();
    return (index?.notes ?? []).filter((memory) => {
      if (memoryType !== "all" && memory.memoryType !== memoryType) return false;
      if (status !== "all" && memory.status !== status) return false;
      if (scope !== "all" && memory.scope !== scope) return false;
      if (archive === "active" && memory.archived) return false;
      if (archive === "archived" && !memory.archived) return false;
      if (!search) return true;
      return [memoryText(t, memory, "name"), memoryText(t, memory, "description"), memory.slug, memory.storeId, ...memory.tags]
        .some((value) => value.toLowerCase().includes(search));
    });
  }, [archive, index?.notes, memoryType, query, scope, status, t]);

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
  }, [selected?.archived, selected?.etag, selected?.slug, selected?.storePath]);

  useEffect(() => {
    setTagNotice(null);
    setTagInput("");
  }, [selectedKey]);

  useEffect(() => {
    if (tagActor && !detail?.writers.includes(tagActor)) setTagActor("");
  }, [detail?.writers, tagActor]);

  async function mutateTag(tag: string, action: "add" | "remove") {
    if (!detail || detail.summary.archived || tagBusy) return;
    if (!tagActor) {
      setTagNotice({ state: "error", text: t("memoryTags.chooseBeforeEdit") });
      return;
    }
    const targetKey = memoryKey(detail.summary);
    setTagBusy(true);
    setTagNotice(null);
    try {
      const result = await changeMemoryTag({
        storePath: detail.summary.storePath,
        slug: detail.summary.slug,
        tag,
        action,
        actor: tagActor,
        ifMatch: detail.summary.etag,
      });
      await onRefreshIndex();
      setDetail((current) => current && memoryKey(current.summary) === targetKey ? {
        ...current,
        summary: {
          ...current.summary,
          tags: result.tags,
          revision: result.revision,
          etag: result.etag,
        },
      } : current);
      if (action === "add") setTagInput("");
      setTagNotice({
        state: "ok",
        text: result.changed
          ? t(action === "add" ? "memoryTags.added" : "memoryTags.removed", { revision: result.revision })
          : t("memoryTags.current"),
      });
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      if (message.includes("etag conflict")) {
        await onRefreshIndex();
        const refreshed = await getMemoryDetail(detail.summary.storePath, detail.summary.slug, false);
        setDetail((current) => current && memoryKey(current.summary) === targetKey ? refreshed : current);
        setTagNotice({ state: "warning", text: t("memoryTags.conflict") });
      } else {
        setTagNotice({ state: "error", text: message });
      }
    } finally {
      setTagBusy(false);
    }
  }

  function submitTag(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (tagInput.trim()) void mutateTag(tagInput, "add");
  }

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
        <span className="memory-row__top"><strong>{memoryText(t, memory, "name")}</strong><time>{memory.updated}</time></span>
        <span className="memory-row__excerpt">{memoryText(t, memory, "description")}</span>
        <span className="memory-row__meta">
          <span>{memory.storeId}</span>
          <span>{t(`memories.types.${memory.memoryType}`)}</span>
          <span>{memory.archived ? t("memories.statuses.archived") : t(`memories.scopes.${memory.scope}`)}</span>
        </span>
      </button>
    ));
  }

  return (
    <section className="view memories-view" aria-labelledby="memories-heading">
      <div className="section-heading">
        <div>
          <h2 id="memories-heading">{t("memories.heading")}</h2>
          <p>{t("memories.description")}</p>
        </div>
        <span className="count-label">{t("common.notesCount", { count: memories.length })}</span>
      </div>
      <div className="memory-filters" aria-label={t("memories.filters")}>
        <label>{t("memories.type")}<select aria-label={t("memories.typeLabel")} value={memoryType} onChange={(event) => setMemoryType(event.target.value)}><option value="all">{t("common.all")}</option><option value="user">{t("memories.types.user")}</option><option value="feedback">{t("memories.types.feedback")}</option><option value="project">{t("memories.types.project")}</option><option value="reference">{t("memories.types.reference")}</option></select></label>
        <label>{t("memories.status")}<select aria-label={t("memories.statusLabel")} value={status} onChange={(event) => setStatus(event.target.value)}><option value="all">{t("common.all")}</option><option value="active">{t("memories.statuses.active")}</option><option value="archived">{t("memories.statuses.archived")}</option></select></label>
        <label>{t("memories.scope")}<select aria-label={t("memories.scopeLabel")} value={scope} onChange={(event) => setScope(event.target.value)}><option value="all">{t("common.all")}</option><option value="global">{t("memories.scopes.global")}</option><option value="repo">{t("memories.scopes.repo")}</option></select></label>
        <label>{t("memories.archive")}<select aria-label={t("memories.archiveLabel")} value={archive} onChange={(event) => setArchive(event.target.value)}><option value="all">{t("common.all")}</option><option value="active">{t("memories.activeOnly")}</option><option value="archived">{t("memories.archivedOnly")}</option></select></label>
      </div>
      {(error || (index?.issues.length ?? 0) > 0) && (
        <div className="access-notice memory-issue" role="alert" title={index?.issues.map((issue) => `${issue.slug ?? issue.storePath}: ${issue.message}`).join("\n")}>
          <CircleAlert aria-hidden="true" size={15} />
          {error ?? t("memories.unreadable", { count: index?.issues.length ?? 0 })}
        </div>
      )}
      <div className="memory-workbench">
        <div className="memory-list" role="listbox" aria-label={t("memories.list")}>
          {globalMemories.length > 0 && <div className="memory-group" role="group" aria-label={t("memories.globalGroup")}><h3>{t("memories.globalGroup")}</h3>{rows(globalMemories)}</div>}
          {projectMemories.length > 0 && <div className="memory-group" role="group" aria-label={t("memories.projectsGroup")}><h3>{t("memories.projectsGroup")}</h3>{rows(projectMemories)}</div>}
          {memories.length === 0 && <EmptyState label={t(loading ? "memories.readingStores" : "memories.noMatch")} />}
        </div>
        <article className="memory-detail" aria-live="polite">
          {detail ? (
            <>
              <div className="memory-detail__head">
                <div>
                  <span>{t(`store.${detail.summary.storeKind}`)} / {detail.summary.storeId}</span>
                  <h3>{memoryText(t, detail.summary, "name")}</h3>
                </div>
                <code>{detail.summary.slug}</code>
              </div>
              <div className="memory-detail__meta">
                <span>{t(`memories.types.${detail.summary.memoryType}`)}</span>
                <span>{t(`memories.statuses.${detail.summary.status}`)}</span>
                <span>{t(`memories.scopes.${detail.summary.scope}`)}</span>
                <span>r{detail.summary.revision}</span>
              </div>
              <p>{memoryText(t, detail.summary, "description")}</p>
              <pre className="memory-body">{mockMemoryKey(detail.summary) ? t(`mock.${mockMemoryKey(detail.summary)}.body`, { defaultValue: detail.body }) : detail.body}</pre>
              <div className="memory-tags">
                <div className="memory-tags__head">
                  <h4>{t("memoryTags.title")}</h4>
                  {!detail.summary.archived && (
                    <label className="tag-actor-picker">
                      <ShieldCheck aria-hidden="true" size={14} />
                      <span className="sr-only">{t("memoryTags.writerIdentity")}</span>
                      <select aria-label={t("memoryTags.writerIdentity")} value={tagActor} onChange={(event) => setTagActor(event.target.value)}>
                        <option value="">{t("memoryTags.chooseWriter")}</option>
                        {detail.writers.map((writer) => <option key={writer} value={writer}>{writer}</option>)}
                      </select>
                    </label>
                  )}
                </div>
                <div className="tag-line" aria-label={t("memoryTags.title")}>
                  {detail.summary.tags.map((tag) => detail.summary.archived ? (
                    <span key={tag}>{tag}</span>
                  ) : (
                    <button
                      key={tag}
                      type="button"
                      disabled={tagBusy || !tagActor}
                      aria-label={t("memoryTags.remove", { tag })}
                      title={t("memoryTags.removeTitle", { tag })}
                      onClick={() => void mutateTag(tag, "remove")}
                    >
                      <span>{tag}</span><X aria-hidden="true" size={12} />
                    </button>
                  ))}
                  {detail.summary.tags.length === 0 && <span className="tag-empty">{t("memoryTags.none")}</span>}
                </div>
                {detail.summary.archived ? (
                  <p className="tag-readonly">{t("memoryTags.readOnly")}</p>
                ) : (
                  <form className="tag-editor" onSubmit={submitTag}>
                    <label>
                      <Tags aria-hidden="true" size={15} />
                      <span className="sr-only">{t("memoryTags.new")}</span>
                      <input
                        aria-label={t("memoryTags.new")}
                        value={tagInput}
                        placeholder={t("memoryTags.addPlaceholder")}
                        maxLength={64}
                        disabled={tagBusy}
                        onChange={(event) => setTagInput(event.target.value)}
                      />
                    </label>
                    <button className="icon-button" type="submit" aria-label={t(tagBusy ? "memoryTags.adding" : "memoryTags.add")} aria-busy={tagBusy} data-state={tagBusy ? "loading" : undefined} title={t(tagBusy ? "memoryTags.adding" : "memoryTags.add")} disabled={tagBusy || !tagActor || !tagInput.trim()}>
                      {tagBusy ? <RefreshCw aria-hidden="true" size={16} /> : <Plus aria-hidden="true" size={16} />}
                    </button>
                  </form>
                )}
                {tagNotice && (
                  <div className="access-notice tag-notice" data-state={tagNotice.state} role={tagNotice.state === "error" ? "alert" : "status"}>
                    {tagNotice.state === "ok" ? <Check aria-hidden="true" size={14} /> : <CircleAlert aria-hidden="true" size={14} />}
                    <span>{tagNotice.text}</span>
                  </div>
                )}
              </div>
            </>
          ) : detailError ? (
            <div className="memory-detail-error" role="alert"><CircleAlert aria-hidden="true" size={18} /><p>{detailError}</p></div>
          ) : (
            <EmptyState label={t(selected ? "memories.readingDetail" : "memories.select")} />
          )}
        </article>
      </div>
    </section>
  );
}

function TagsView({
  query,
  index,
  onOpenTag,
}: {
  query: string;
  index: MemoryIndexPayload | null;
  onOpenTag: (tag: string) => void;
}) {
  const { t } = useTranslation();
  const tags = useMemo(() => {
    const counts = new Map<string, { count: number; scopes: Set<"global" | "project"> }>();
    for (const memory of index?.notes ?? []) {
      for (const name of memory.tags) {
        const current = counts.get(name) ?? { count: 0, scopes: new Set<"global" | "project">() };
        current.count += 1;
        current.scopes.add(memory.storeKind);
        counts.set(name, current);
      }
    }
    const search = query.trim().toLowerCase();
    return [...counts.entries()]
      .map(([name, value]) => ({
        name,
        count: value.count,
        scope: value.scopes.size > 1 ? "mixed" : [...value.scopes][0],
      }))
      .filter((tag) => tag.name.includes(search))
      .sort((left, right) => left.name.localeCompare(right.name));
  }, [index?.notes, query]);
  return (
    <section className="view tags-view" aria-labelledby="tags-heading">
      <div className="section-heading">
        <div><h2 id="tags-heading">{t("tagIndex.heading")}</h2><p>{t("tagIndex.description")}</p></div>
        <span className="count-label">{t("common.tagsCount", { count: tags.length })}</span>
      </div>
      <div className="tag-table" role="table" aria-label={t("tagIndex.table")}>
        <div className="tag-table__head" role="row">
          <span role="columnheader">{t("tagIndex.tag")}</span><span role="columnheader">{t("tagIndex.scope")}</span><span role="columnheader">{t("tagIndex.notes")}</span><span />
        </div>
        {tags.map((tag) => (
          <div className="tag-table__row" role="row" key={tag.name}>
            <strong role="cell">{tag.name}</strong>
            <span role="cell">{t(`tagIndex.scopes.${tag.scope}`)}</span>
            <span role="cell" className="mono-number">{tag.count}</span>
            <button className="icon-button" type="button" aria-label={t("tagIndex.open", { name: tag.name })} title={t("tagIndex.open", { name: tag.name })} onClick={() => onOpenTag(tag.name)}>
              <ChevronRight aria-hidden="true" size={17} />
            </button>
          </div>
        ))}
        {tags.length === 0 && <EmptyState label={t("tagIndex.noMatch")} />}
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
  const { t } = useTranslation();
  const [projectPath, setProjectPath] = useState("");

  async function submitProject(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (await onRegister(projectPath)) setProjectPath("");
  }

  return (
    <section className="view settings-view" aria-labelledby="settings-heading">
      <div className="section-heading">
        <div><h2 id="settings-heading">{t("settings.runtime")}</h2><p>{t("settings.description")}</p></div>
      </div>
      <dl className="settings-list">
        <div><dt>{t("settings.application")}</dt><dd>Momonogi Desktop</dd></div>
        <div><dt>{t("settings.version")}</dt><dd><code>{bootstrap?.appVersion ?? t("common.loading")}</code></dd></div>
        <div><dt>{t("settings.coreSchema")}</dt><dd><code>v{bootstrap?.coreSchema ?? "-"}</code></dd></div>
        <div><dt>{t("settings.bridge")}</dt><dd><span className="runtime-badge">{bootstrap?.bridge ? t(`bridge.${bootstrap.bridge}`) : t("common.loading")}</span></dd></div>
      </dl>

      <div className="section-heading store-registry-heading">
        <div><h2>{t("settings.registry")}</h2><p>{t("settings.registryDescription")}</p></div>
        <span className="count-label">{t("common.storesCount", { count: stores.length })}</span>
      </div>
      <form className="store-register" onSubmit={(event) => void submitProject(event)}>
        <label>
          <span className="sr-only">{t("settings.projectPath")}</span>
          <FolderPlus aria-hidden="true" size={17} />
          <input
            value={projectPath}
            aria-label={t("settings.projectPath")}
            placeholder="/path/to/project/.momonogi"
            onChange={(event) => setProjectPath(event.target.value)}
          />
        </label>
        <button className="secondary-button" type="submit" aria-busy={registryBusy} data-state={registryBusy ? "loading" : undefined} disabled={registryBusy || !projectPath.trim()}>
          {t("settings.register")}
        </button>
      </form>
      {registryError && <div className="access-notice registry-notice" role="alert"><CircleAlert aria-hidden="true" size={15} />{registryError}</div>}
      <div className="store-list" role="list" aria-label={t("settings.registered")}>
        {stores.map((store) => (
          <article className="store-row" role="listitem" key={`${store.kind}:${store.path}`}>
            <Database aria-hidden="true" size={18} />
            <div className="store-row__identity">
              <h3>{store.storeId ?? t(store.kind === "global" ? "settings.globalStore" : "settings.projectStore")}</h3>
              <code title={store.path}>{store.path}</code>
            </div>
            <span className="store-health" data-health={store.health}>{t(`store.${store.health}`)}</span>
            <code className="store-revision">r{store.revision ?? "-"}</code>
            {store.kind === "project" ? (
              <button
                className="icon-button"
                type="button"
                aria-label={t("common.remove", { name: store.storeId ?? store.path })}
                title={t("settings.removeRegistry")}
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
  const { t } = useTranslation();
  const [active, setActive] = useState<ViewId>("agents");
  const [query, setQuery] = useState("");
  const [bootstrap, setBootstrap] = useState<BootstrapPayload | null>(null);
  const [agentDiscovery, setAgentDiscovery] = useState<AgentDiscoveryPayload | null>(null);
  const [discoveryError, setDiscoveryError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [savingAgentId, setSavingAgentId] = useState<string | null>(null);
  const [accessNotice, setAccessNotice] = useState<AccessNotice | null>(null);
  const [configurationPlan, setConfigurationPlan] = useState<ConfigurationPlanPayload | null>(null);
  const [configurationNotice, setConfigurationNotice] = useState<AccessNotice | null>(null);
  const [configurationBusy, setConfigurationBusy] = useState(false);
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

  const loadConfigurationPreview = useCallback(async (agentId: string) => {
    setConfigurationBusy(true);
    setConfigurationNotice(null);
    try {
      const plan = await previewAgentConfiguration(agentId);
      setConfigurationPlan(plan);
      if (!plan) {
        setConfigurationNotice({ state: "warning", text: t("config.noAdapter") });
      }
      return plan;
    } catch (cause) {
      setConfigurationPlan(null);
      setConfigurationNotice({ state: "error", text: cause instanceof Error ? cause.message : String(cause) });
      return null;
    } finally {
      setConfigurationBusy(false);
    }
  }, [t]);

  const updateAccess = useCallback(async (agentId: string, role: AgentRole, actor: string) => {
    const ifMatch = agentDiscovery?.storeEtag;
    if (!ifMatch) {
      setAccessNotice({ state: "error", text: t("agents.noManifest") });
      return;
    }
    setSavingAgentId(agentId);
    setAccessNotice(null);
    try {
      const result = await setAgentAccess({ agentId, role, actor, ifMatch });
      await refreshData();
      await loadConfigurationPreview(agentId);
      setAccessNotice({
        state: "ok",
        text: result.changed ? t("agents.updateRevision", { revision: result.revision }) : t("agents.alreadyCurrent"),
      });
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      if (message.includes("etag conflict")) {
        await refreshData();
        setAccessNotice({ state: "warning", text: t("agents.conflict") });
      } else {
        setAccessNotice({ state: "error", text: message });
      }
    } finally {
      setSavingAgentId(null);
    }
  }, [agentDiscovery?.storeEtag, loadConfigurationPreview, refreshData, t]);

  const applyConfiguration = useCallback(async () => {
    if (!configurationPlan) return;
    setConfigurationBusy(true);
    setConfigurationNotice(null);
    try {
      const result = await applyAgentConfiguration(configurationPlan.agentId, configurationPlan.digest);
      await refreshData();
      setConfigurationPlan(await previewAgentConfiguration(configurationPlan.agentId));
      setConfigurationNotice({
        state: "ok",
        text: result.changedFiles.length === 0
          ? t("config.alreadyCurrent")
          : t("config.synchronized", { count: result.changedFiles.length }),
      });
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      if (message.includes("preview is stale") || message.includes("configuration conflict")) {
        setConfigurationPlan(await previewAgentConfiguration(configurationPlan.agentId));
        setConfigurationNotice({ state: "warning", text: t("config.stale") });
      } else {
        setConfigurationNotice({ state: "error", text: message });
      }
    } finally {
      setConfigurationBusy(false);
    }
  }, [configurationPlan, refreshData, t]);

  const revealStoreFolder = useCallback(async (path: string) => {
    setDiscoveryError(null);
    try {
      await openStoreFolder(path);
    } catch (cause) {
      setDiscoveryError(cause instanceof Error ? cause.message : String(cause));
    }
  }, []);

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
        configurationNotice={configurationNotice}
        configurationPlan={configurationPlan}
        configurationBusy={configurationBusy}
        savingAgentId={savingAgentId}
        onSetAccess={(agentId, role, actor) => void updateAccess(agentId, role, actor)}
        onPreviewConfiguration={(agentId) => void loadConfigurationPreview(agentId)}
        onApplyConfiguration={() => void applyConfiguration()}
        onDismissConfiguration={() => { setConfigurationPlan(null); setConfigurationNotice(null); }}
        onOpenStoreFolder={(path) => void revealStoreFolder(path)}
      />
    );
    if (active === "memories") return (
      <MemoriesView
        query={query}
        index={memoryIndex}
        loading={refreshing && !memoryIndex}
        error={memoryError}
        onRefreshIndex={async () => setMemoryIndex(await getMemoryIndex())}
      />
    );
    if (active === "tags") return <TagsView query={query} index={memoryIndex} onOpenTag={(tag) => { setActive("memories"); setQuery(tag); }} />;
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
  }, [accessNotice, active, addProjectStore, agentDiscovery, applyConfiguration, bootstrap, configurationBusy, configurationNotice, configurationPlan, discoveryError, loadConfigurationPreview, memoryError, memoryIndex, query, refreshing, registryBusy, registryError, removeStore, revealStoreFolder, savingAgentId, stores, updateAccess]);

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
          <span><span className="status-dot" aria-hidden="true" />{t(`bridge.${bootstrap?.bridge ?? "browser"}`)}</span>
          <span>{t("store.schema")} v{bootstrap?.coreSchema ?? "-"}</span>
          <span className="status-line__version">Momonogi {bootstrap?.appVersion ?? t("common.loading")}</span>
        </footer>
      </div>
    </div>
  );
}
