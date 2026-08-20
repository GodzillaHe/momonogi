use crate::store::{self, Error, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const BEGIN: &str = "<!-- BEGIN MOMONOGI -->";
const END: &str = "<!-- END MOMONOGI -->";
const EVENTS: [&str; 3] = ["SessionStart", "UserPromptSubmit", "PreCompact"];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Host {
    Codex,
    Claude,
    Opencode,
    Openclaw,
}

impl Host {
    pub fn agent_id(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude-code",
            Self::Opencode => "opencode",
            Self::Openclaw => "openclaw",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Claude => "Claude Code",
            Self::Opencode => "OpenCode",
            Self::Openclaw => "OpenClaw",
        }
    }

    pub fn from_agent_id(agent_id: &str) -> Option<Self> {
        match agent_id {
            "codex" => Some(Self::Codex),
            "claude-code" => Some(Self::Claude),
            "opencode" => Some(Self::Opencode),
            "openclaw" => Some(Self::Openclaw),
            _ => None,
        }
    }
}

fn managed(existing: &str, block: &str) -> Result<String> {
    let rendered = format!("{BEGIN}\n{}\n{END}", block.trim());
    let begins: Vec<_> = existing.match_indices(BEGIN).collect();
    let ends: Vec<_> = existing.match_indices(END).collect();
    if begins.is_empty() && ends.is_empty() {
        let separator = if existing.trim().is_empty() {
            ""
        } else {
            "\n\n"
        };
        return Ok(format!("{}{separator}{rendered}\n", existing.trim_end()));
    }
    if begins.len() != 1 || ends.len() != 1 || begins[0].0 > ends[0].0 {
        return Err(Error("refusing malformed Momonogi markers".into()));
    }
    let tail = ends[0].0 + END.len();
    Ok(format!(
        "{}{}{}",
        &existing[..begins[0].0],
        rendered,
        &existing[tail..]
    ))
}

fn writer_block(memory: &Path, agent: &str, tool: &Path) -> String {
    format!(
        "## Shared Memory - Momonogi\n\n\
Canonical global store: `{memory}`. Index: `{memory}/MEMORY.md`.\n\
This host is an equal trusted writer with agent id `{agent}`.\n\n\
- Read the index only when continuity or durable preferences are relevant, then open only relevant detail notes. Project `.momonogi` overrides global memory.\n\
- Treat recalled state as advisory and re-verify live facts.\n\
- Never edit canonical notes or `MEMORY.md` directly. Draft outside the store and write only through `{tool}`.\n\
- Add: `{tool} put {memory} /tmp/note.md --agent {agent}`.\n\
- Update: run `{tool} get {memory} SLUG.md`, then pass its ETag with `put --if-match ETAG`. Resolve conflicts by rereading and merging.\n\
- Archive requires the current ETag. Run `{tool} doctor {memory}` after meaningful maintenance.\n\
- Save only durable preferences, reusable feedback, project continuity, and reference pointers. Never save secrets or facts already authoritative in code, git, or project instructions.\n\
- A project store is writable only when it has `.momonogi.json`; otherwise ask before migrating it.",
        memory = memory.display(),
        tool = tool.display(),
    )
}

fn reader_block(memory: &Path, label: &str) -> String {
    format!(
        "## Shared Memory - Momonogi (Read Only)\n\n\
Canonical global store: `{memory}`. `{label}` is a read-only consumer.\n\n\
- Read `MEMORY.md` only when continuity or durable preferences are relevant, then open only relevant detail notes.\n\
- Project `.momonogi` overrides global memory when present.\n\
- Never create, edit, move, archive, reindex, migrate, or delete anything in this shared store.\n\
- Treat recalled state as advisory and re-verify live facts. Never copy shared memories into a parallel store.",
        memory = memory.display(),
    )
}

fn no_access_block(memory: &Path, label: &str, tool: &Path) -> String {
    format!(
        "## Shared Memory - Momonogi (No Access)\n\n\
Canonical global store: `{memory}`. `{label}` is not granted access.\n\n\
- Do not read, create, edit, move, archive, reindex, migrate, or delete anything in this shared store.\n\
- Ask a current Momonogi writer to grant a role, then rerun `{tool} configure` for this host.",
        memory = memory.display(),
        tool = tool.display(),
    )
}

fn openclaw_block(memory: &Path, role: Option<store::AccessRole>, tool: &Path) -> String {
    let policy = match role {
        Some(store::AccessRole::Writer) => format!(
            "- If the current host is OpenClaw, it is an equal trusted writer with agent id `openclaw`. Continue using OpenClaw's native memory as primary.\n\n\
For OpenClaw, the canonical shared store is `{memory}`. Index: `{memory}/MEMORY.md`.\n\n\
- Read the index only when continuity or durable preferences are relevant, then open only relevant detail notes. Project `.momonogi` overrides global memory.\n\
- Never edit canonical notes or `MEMORY.md` directly; write only through `{tool}` with `--agent openclaw`.\n\
- Use ETags for updates and archives, and run `{tool} doctor {memory}` after meaningful maintenance.\n\
- Save only durable preferences, reusable feedback, project continuity, and reference pointers. Never save secrets or facts already authoritative elsewhere.",
            memory = memory.display(),
            tool = tool.display(),
        ),
        Some(store::AccessRole::Reader) => format!(
            "- If the current host is OpenClaw, it is a read-only Momonogi consumer. Continue using OpenClaw's native memory as primary and use the shared store only as supplementary context.\n\n\
For OpenClaw, the canonical shared store is `{memory}`.\n\n\
- Read `MEMORY.md` only when continuity or durable preferences are relevant, then open only relevant detail notes.\n\
- Project `.momonogi` overrides global memory when present.\n\
- Never create, edit, move, archive, reindex, migrate, or delete anything in this shared store.\n\
- Treat recalled state as advisory and re-verify live facts. Never copy shared memories into OpenClaw's native memory.",
            memory = memory.display(),
        ),
        None => format!(
            "- If the current host is OpenClaw, it is not granted access to the shared Momonogi store. Continue using OpenClaw's native memory only.\n\n\
For OpenClaw, the canonical shared store is `{memory}`. Do not read, create, edit, move, archive, reindex, migrate, or delete anything in it. Ask a current writer to grant access, then rerun `{tool} configure --host openclaw`.",
            memory = memory.display(),
            tool = tool.display(),
        ),
    };
    format!(
        "## Shared Memory - Momonogi (Host Conditional)\n\n\
This workspace may also be opened by Codex or another agent host.\n\
- If the current host is not OpenClaw, this block does not change its Momonogi role; follow that host's global rules.\n\
{policy}",
    )
}

fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| Error("cannot determine home directory".into()))
}

fn target(host: Host, home: &Path, openclaw_workspace: Option<&Path>) -> Result<PathBuf> {
    Ok(match host {
        Host::Codex => home.join(".codex/AGENTS.md"),
        Host::Claude => home.join(".claude/CLAUDE.md"),
        Host::Opencode => home.join(".config/opencode/AGENTS.md"),
        Host::Openclaw => openclaw_workspace
            .ok_or_else(|| Error("openclaw requires --openclaw-workspace".into()))?
            .join("AGENTS.md"),
    })
}

fn refuse_symlink(path: &Path) -> Result<()> {
    if path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        Err(Error(format!(
            "refusing to replace symlinked configuration: {}",
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn read_optional(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(value),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(Error(format!("cannot read {}: {error}", path.display()))),
    }
}

fn status(event: &str) -> String {
    format!("Momonogi: {event} continuity (managed v1)")
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn hook_command(memory: &Path, mode: &str, tool: &Path) -> String {
    [
        shell_quote(&tool.to_string_lossy()),
        "hook".into(),
        "--memory-root".into(),
        shell_quote(&memory.to_string_lossy()),
        "--mode".into(),
        shell_quote(mode),
    ]
    .join(" ")
}

fn hook_config(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    refuse_symlink(path)?;
    let value: Value = serde_json::from_str(&fs::read_to_string(path)?).map_err(|error| {
        Error(format!(
            "invalid hook configuration {}: {error}",
            path.display()
        ))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        Error(format!(
            "hook configuration must be an object: {}",
            path.display()
        ))
    })
}

fn merged_hooks(path: &Path, memory: &Path, mode: &str, tool: &Path) -> Result<String> {
    let mut root = hook_config(path)?;
    let hooks = root
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| Error(format!("hooks must be an object: {}", path.display())))?;
    let command = hook_command(memory, mode, tool);
    for event in EVENTS {
        let groups = hooks
            .entry(event)
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or_else(|| {
                Error(format!(
                    "hooks.{event} must be an array: {}",
                    path.display()
                ))
            })?;
        let managed = status(event);
        for group in groups.iter() {
            let handlers = group
                .get("hooks")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    Error(format!(
                        "hooks.{event} entries must contain a hooks array: {}",
                        path.display()
                    ))
                })?;
            if handlers.iter().any(|handler| !handler.is_object()) {
                return Err(Error(format!(
                    "hooks.{event} handlers must be objects: {}",
                    path.display()
                )));
            }
        }
        for group in groups.iter_mut() {
            if let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                handlers.retain(|handler| {
                    handler.get("statusMessage").and_then(Value::as_str) != Some(&managed)
                });
            }
        }
        groups.retain(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .is_some_and(|handlers| !handlers.is_empty())
        });
        let handler = json!({
            "type": "command",
            "command": command,
            "timeout": 10,
            "statusMessage": managed,
        });
        let mut group = Map::new();
        group.insert("hooks".into(), Value::Array(vec![handler]));
        if event == "SessionStart" {
            group.insert(
                "matcher".into(),
                Value::String("startup|resume|clear|compact".into()),
            );
        }
        groups.push(Value::Object(group));
    }
    let mut rendered = serde_json::to_string_pretty(&Value::Object(root))?;
    rendered.push('\n');
    Ok(rendered)
}

fn hooks_without_momonogi(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut root = hook_config(path)?;
    let mut changed = false;
    let mut remove_hooks_key = false;
    if let Some(value) = root.get_mut("hooks") {
        let hooks = value
            .as_object_mut()
            .ok_or_else(|| Error(format!("hooks must be an object: {}", path.display())))?;
        let mut empty_managed_events = Vec::new();
        for event in EVENTS {
            let Some(value) = hooks.get_mut(event) else {
                continue;
            };
            let groups = value.as_array_mut().ok_or_else(|| {
                Error(format!(
                    "hooks.{event} must be an array: {}",
                    path.display()
                ))
            })?;
            let managed = status(event);
            let mut event_changed = false;
            for group in groups.iter_mut() {
                let handlers = group
                    .get_mut("hooks")
                    .and_then(Value::as_array_mut)
                    .ok_or_else(|| {
                        Error(format!(
                            "hooks.{event} entries must contain a hooks array: {}",
                            path.display()
                        ))
                    })?;
                let before = handlers.len();
                handlers.retain(|handler| {
                    handler.get("statusMessage").and_then(Value::as_str) != Some(&managed)
                });
                event_changed |= handlers.len() != before;
            }
            let before = groups.len();
            groups.retain(|group| {
                group
                    .get("hooks")
                    .and_then(Value::as_array)
                    .is_some_and(|handlers| !handlers.is_empty())
            });
            event_changed |= groups.len() != before;
            changed |= event_changed;
            if event_changed && groups.is_empty() {
                empty_managed_events.push(event);
            }
        }
        for event in empty_managed_events {
            hooks.remove(event);
        }
        remove_hooks_key = changed && hooks.is_empty();
    }
    if remove_hooks_key {
        root.remove("hooks");
    }
    if changed {
        let mut rendered = serde_json::to_string_pretty(&Value::Object(root))?;
        rendered.push('\n');
        Ok(Some(rendered))
    } else {
        Ok(None)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationFileKind {
    Rules,
    Hooks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationAction {
    Create,
    Update,
    RemoveManaged,
    Unchanged,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationFile {
    pub path: String,
    pub kind: ConfigurationFileKind,
    pub action: ConfigurationAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_hash: Option<String>,
    pub after_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationPlan {
    pub agent_id: String,
    pub role: Option<store::AccessRole>,
    pub files: Vec<ConfigurationFile>,
    pub warnings: Vec<String>,
    pub digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationApply {
    pub changed_files: Vec<String>,
    pub digest: String,
}

pub struct SyncOptions<'a> {
    pub host: Host,
    pub home: &'a Path,
    pub openclaw_workspaces: &'a [PathBuf],
    pub codex_projects: &'a [PathBuf],
    pub memory_root: &'a Path,
    pub hook_mode: &'a str,
    pub install_hooks: bool,
    pub tool: &'a Path,
}

struct PlannedFile {
    summary: ConfigurationFile,
    contents: Vec<u8>,
    mode: u32,
}

fn canonical_directories(paths: &[PathBuf], label: &str) -> Result<Vec<PathBuf>> {
    let mut directories = BTreeSet::new();
    for path in paths {
        let expanded = store::expand_path(path);
        let canonical = expanded
            .canonicalize()
            .map_err(|error| Error(format!("invalid {label} {}: {error}", expanded.display())))?;
        if !canonical.is_dir() {
            return Err(Error(format!(
                "invalid {label} {}: not a directory",
                canonical.display()
            )));
        }
        directories.insert(canonical);
    }
    Ok(directories.into_iter().collect())
}

fn plan_file(
    path: PathBuf,
    kind: ConfigurationFileKind,
    contents: String,
    mode: u32,
    removes_managed: bool,
) -> Result<PlannedFile> {
    refuse_symlink(&path)?;
    let existed = path.exists();
    let before = read_optional(&path)?;
    let action = if before == contents {
        ConfigurationAction::Unchanged
    } else if !existed {
        ConfigurationAction::Create
    } else if removes_managed {
        ConfigurationAction::RemoveManaged
    } else {
        ConfigurationAction::Update
    };
    Ok(PlannedFile {
        summary: ConfigurationFile {
            path: path.to_string_lossy().into_owned(),
            kind,
            action,
            before_hash: existed.then(|| store::sha(before.as_bytes())),
            after_hash: store::sha(contents.as_bytes()),
        },
        contents: contents.into_bytes(),
        mode,
    })
}

fn current_hash(path: &Path) -> Result<Option<String>> {
    refuse_symlink(path)?;
    match fs::read(path) {
        Ok(contents) => Ok(Some(store::sha(contents))),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error(format!("cannot read {}: {error}", path.display()))),
    }
}

fn build_plan(options: &SyncOptions<'_>) -> Result<(ConfigurationPlan, Vec<PlannedFile>)> {
    let memory = store::expand_path(options.memory_root);
    let memory = memory.canonicalize().map_err(|error| {
        Error(format!(
            "invalid Momonogi store {}: {error}",
            memory.display()
        ))
    })?;
    let manifest = store::read_manifest(&memory)?;
    let role = store::manifest_role(&manifest, options.host.agent_id());
    let workspaces = canonical_directories(options.openclaw_workspaces, "OpenClaw workspace")?;
    let codex_projects = canonical_directories(options.codex_projects, "Codex project")?;
    let mut warnings = Vec::new();
    let mut planned = Vec::new();

    let rule_targets = if options.host == Host::Openclaw {
        if workspaces.is_empty() {
            warnings
                .push("No registered project workspace is available for OpenClaw rules.".into());
        }
        workspaces
            .iter()
            .map(|workspace| target(options.host, options.home, Some(workspace)))
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![target(options.host, options.home, None)?]
    };
    for path in rule_targets {
        let existing = read_optional(&path)?;
        let block = match (options.host, role) {
            (Host::Openclaw, role) => openclaw_block(&memory, role, options.tool),
            (_, Some(store::AccessRole::Writer)) => {
                writer_block(&memory, options.host.agent_id(), options.tool)
            }
            (_, Some(store::AccessRole::Reader)) => reader_block(&memory, options.host.label()),
            (_, None) => no_access_block(&memory, options.host.label(), options.tool),
        };
        planned.push(plan_file(
            path,
            ConfigurationFileKind::Rules,
            managed(&existing, &block)?,
            0o644,
            false,
        )?);
    }

    if options.install_hooks && options.host == Host::Claude {
        let path = options.home.join(".claude/settings.json");
        if role == Some(store::AccessRole::Writer) {
            let rendered = merged_hooks(&path, &memory, options.hook_mode, options.tool)?;
            planned.push(plan_file(
                path,
                ConfigurationFileKind::Hooks,
                rendered,
                0o600,
                false,
            )?);
        } else if let Some(rendered) = hooks_without_momonogi(&path)? {
            planned.push(plan_file(
                path,
                ConfigurationFileKind::Hooks,
                rendered,
                0o600,
                true,
            )?);
        }
    }

    if options.install_hooks && options.host == Host::Codex {
        if codex_projects.is_empty() {
            warnings
                .push("No registered Codex project is available for project-scoped hooks.".into());
        }
        for project in codex_projects {
            let path = project.join(".codex/hooks.json");
            if role == Some(store::AccessRole::Writer) {
                let rendered = merged_hooks(&path, &memory, options.hook_mode, options.tool)?;
                planned.push(plan_file(
                    path,
                    ConfigurationFileKind::Hooks,
                    rendered,
                    0o600,
                    false,
                )?);
            } else if let Some(rendered) = hooks_without_momonogi(&path)? {
                planned.push(plan_file(
                    path,
                    ConfigurationFileKind::Hooks,
                    rendered,
                    0o600,
                    true,
                )?);
            }
        }
    }

    let files: Vec<_> = planned.iter().map(|file| file.summary.clone()).collect();
    let digest = store::sha(serde_json::to_vec(&(
        options.host,
        role,
        &files,
        &warnings,
    ))?);
    Ok((
        ConfigurationPlan {
            agent_id: options.host.agent_id().into(),
            role,
            files,
            warnings,
            digest,
        },
        planned,
    ))
}

fn apply_files(files: &[PlannedFile]) -> Result<Vec<String>> {
    for file in files {
        let path = Path::new(&file.summary.path);
        let actual = current_hash(path)?;
        if actual != file.summary.before_hash {
            return Err(Error(format!(
                "configuration conflict for {}: the file changed after preview",
                path.display()
            )));
        }
    }
    let mut changed = Vec::new();
    for file in files {
        if file.summary.action == ConfigurationAction::Unchanged {
            continue;
        }
        let path = Path::new(&file.summary.path);
        store::atomic_write(path, &file.contents, file.mode)?;
        changed.push(file.summary.path.clone());
    }
    Ok(changed)
}

pub fn preview_host(options: &SyncOptions<'_>) -> Result<ConfigurationPlan> {
    build_plan(options).map(|(plan, _)| plan)
}

pub fn apply_host(options: &SyncOptions<'_>, expected_digest: &str) -> Result<ConfigurationApply> {
    let (plan, files) = build_plan(options)?;
    if plan.digest != expected_digest {
        return Err(Error(
            "configuration preview is stale; refresh it before applying".into(),
        ));
    }
    Ok(ConfigurationApply {
        changed_files: apply_files(&files)?,
        digest: plan.digest,
    })
}

pub fn apply(
    hosts: &[Host],
    workspaces: &[PathBuf],
    codex_projects: &[PathBuf],
    memory_root: &Path,
    hook_mode: &str,
    install_hooks: bool,
) -> Result<Vec<PathBuf>> {
    if hosts.contains(&Host::Openclaw) && workspaces.is_empty() {
        return Err(Error(
            "--host openclaw requires --openclaw-workspace".into(),
        ));
    }
    let home = home()?;
    let tool = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("momo"));
    let mut plans = Vec::new();
    let mut configured = BTreeSet::new();
    for host in hosts {
        let (_, files) = build_plan(&SyncOptions {
            host: *host,
            home: &home,
            openclaw_workspaces: workspaces,
            codex_projects,
            memory_root,
            hook_mode,
            install_hooks,
            tool: &tool,
        })?;
        configured.extend(files.iter().map(|file| PathBuf::from(&file.summary.path)));
        plans.extend(files);
    }
    apply_files(&plans)?;
    Ok(configured.into_iter().collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn managed_block_is_idempotent() {
        let once = super::managed("before\n", "body").unwrap();
        let twice = super::managed(&once, "new body").unwrap();
        assert_eq!(twice.matches(super::BEGIN).count(), 1);
        assert!(twice.contains("new body"));
        assert!(!twice.contains("\nbody\n"));
    }
}
