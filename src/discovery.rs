use crate::store::{self, Manifest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const BEGIN: &str = "<!-- BEGIN MOMONOGI -->";
const END: &str = "<!-- END MOMONOGI -->";
const EVENTS: [&str; 3] = ["SessionStart", "UserPromptSubmit", "PreCompact"];

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProbe {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    Writer,
    Reader,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManagedHookState {
    Active,
    Partial,
    Missing,
    Invalid,
    NotApplicable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredAgent {
    pub id: String,
    pub name: String,
    pub command: String,
    pub role: AgentRole,
    pub installed: bool,
    pub configured: bool,
    pub managed: bool,
    pub hook_state: ManagedHookState,
    pub config_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_issue: Option<String>,
}

pub struct DiscoveryInput<'a> {
    pub home: &'a Path,
    pub path: &'a OsStr,
    pub manifest: Option<&'a Manifest>,
    pub catalog: &'a [AgentProbe],
    pub openclaw_workspaces: &'a [PathBuf],
    pub codex_projects: &'a [PathBuf],
}

struct AdapterPaths {
    rules: Vec<PathBuf>,
    hooks: Vec<PathBuf>,
    supports_hooks: bool,
}

fn adapter_paths(input: &DiscoveryInput<'_>, agent_id: &str) -> AdapterPaths {
    match agent_id {
        "codex" => AdapterPaths {
            rules: vec![input.home.join(".codex/AGENTS.md")],
            hooks: input
                .codex_projects
                .iter()
                .map(|project| project.join(".codex/hooks.json"))
                .collect(),
            supports_hooks: true,
        },
        "claude-code" => AdapterPaths {
            rules: vec![input.home.join(".claude/CLAUDE.md")],
            hooks: vec![input.home.join(".claude/settings.json")],
            supports_hooks: true,
        },
        "opencode" => AdapterPaths {
            rules: vec![input.home.join(".config/opencode/AGENTS.md")],
            hooks: Vec::new(),
            supports_hooks: false,
        },
        "openclaw" => AdapterPaths {
            rules: input
                .openclaw_workspaces
                .iter()
                .map(|workspace| workspace.join("AGENTS.md"))
                .collect(),
            hooks: Vec::new(),
            supports_hooks: false,
        },
        _ => AdapterPaths {
            rules: Vec::new(),
            hooks: Vec::new(),
            supports_hooks: false,
        },
    }
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn valid_command(command: &str) -> bool {
    !command.is_empty() && !command.contains(['/', '\\']) && command != "." && command != ".."
}

fn installed_command(commands: &[String], path: &OsStr) -> Option<String> {
    for command in commands.iter().filter(|command| valid_command(command)) {
        for directory in std::env::split_paths(path) {
            let candidate = directory.join(command);
            if is_executable(&candidate) {
                return Some(command.clone());
            }
            #[cfg(windows)]
            for extension in ["exe", "cmd", "bat"] {
                if is_executable(&candidate.with_extension(extension)) {
                    return Some(command.clone());
                }
            }
        }
    }
    None
}

fn role(manifest: Option<&Manifest>, agent_id: &str) -> AgentRole {
    match manifest.and_then(|manifest| store::manifest_role(manifest, agent_id)) {
        Some(store::AccessRole::Writer) => AgentRole::Writer,
        Some(store::AccessRole::Reader) => AgentRole::Reader,
        None => AgentRole::None,
    }
}

fn inspect_rules(paths: &[PathBuf]) -> (bool, bool, Option<String>) {
    let mut configured = false;
    let mut managed = false;
    for path in paths.iter().filter(|path| path.is_file()) {
        configured = true;
        match fs::read_to_string(path) {
            Ok(contents) => {
                let begins = contents.matches(BEGIN).count();
                let ends = contents.matches(END).count();
                if begins == 1 && ends == 1 {
                    managed = true;
                } else if begins != 0 || ends != 0 {
                    return (
                        configured,
                        managed,
                        Some(format!("malformed Momonogi markers in {}", path.display())),
                    );
                }
            }
            Err(error) => {
                return (
                    configured,
                    managed,
                    Some(format!("cannot read {}: {error}", path.display())),
                );
            }
        }
    }
    (configured, managed, None)
}

fn managed_event(value: &Value, event: &str) -> bool {
    value
        .get("hooks")
        .and_then(|hooks| hooks.get(event))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .any(|handler| {
            handler.get("statusMessage").and_then(Value::as_str)
                == Some(&format!("Momonogi: {event} continuity (managed v1)"))
        })
}

fn inspect_hooks(paths: &[PathBuf], supported: bool) -> (ManagedHookState, Option<String>) {
    if !supported {
        return (ManagedHookState::NotApplicable, None);
    }
    if paths.is_empty() {
        return (ManagedHookState::Missing, None);
    }

    let mut complete = 0;
    let mut managed_events = 0;
    for path in paths {
        if !path.is_file() {
            continue;
        }
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) => {
                return (
                    ManagedHookState::Invalid,
                    Some(format!("cannot read {}: {error}", path.display())),
                );
            }
        };
        let value: Value = match serde_json::from_str(&contents) {
            Ok(value) => value,
            Err(error) => {
                return (
                    ManagedHookState::Invalid,
                    Some(format!(
                        "invalid hook configuration {}: {error}",
                        path.display()
                    )),
                );
            }
        };
        let count = EVENTS
            .iter()
            .filter(|event| managed_event(&value, event))
            .count();
        managed_events += count;
        if count == EVENTS.len() {
            complete += 1;
        }
    }

    let state = if complete == paths.len() {
        ManagedHookState::Active
    } else if managed_events == 0 {
        ManagedHookState::Missing
    } else {
        ManagedHookState::Partial
    };
    (state, None)
}

fn discover_probe(input: &DiscoveryInput<'_>, probe: &AgentProbe) -> DiscoveredAgent {
    let paths = adapter_paths(input, &probe.id);
    let installed_command = installed_command(&probe.commands, input.path);
    let (configured, managed, rules_issue) = inspect_rules(&paths.rules);
    let (hook_state, hook_issue) = inspect_hooks(&paths.hooks, paths.supports_hooks);
    let mut config_paths = paths.rules;
    config_paths.extend(paths.hooks);
    DiscoveredAgent {
        id: probe.id.clone(),
        name: probe.name.clone(),
        command: installed_command
            .clone()
            .or_else(|| probe.commands.first().cloned())
            .unwrap_or_default(),
        role: role(input.manifest, &probe.id),
        installed: installed_command.is_some(),
        configured,
        managed,
        hook_state,
        config_paths: config_paths
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        config_issue: rules_issue.or(hook_issue),
    }
}

pub fn discover_agents(input: DiscoveryInput<'_>) -> Vec<DiscoveredAgent> {
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for probe in input.catalog {
        if !store::valid_agent_id(&probe.id) || !seen.insert(probe.id.clone()) {
            continue;
        }
        let agent = discover_probe(&input, probe);
        if agent.installed || agent.configured || agent.managed || agent.role != AgentRole::None {
            result.push(agent);
        }
    }

    if let Some(manifest) = input.manifest {
        let manifest_agents: BTreeSet<_> = manifest
            .writers
            .iter()
            .chain(&manifest.readers)
            .map(String::as_str)
            .collect();
        for agent_id in manifest_agents {
            if seen.contains(agent_id) {
                continue;
            }
            let probe = AgentProbe {
                id: agent_id.to_owned(),
                name: agent_id.to_owned(),
                commands: Vec::new(),
            };
            result.push(discover_probe(&input, &probe));
        }
    }

    result
}
