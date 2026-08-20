use momonogi::discovery::{
    AgentProbe, AgentRole, DiscoveryInput, ManagedHookState, discover_agents,
};
use momonogi::store::{Manifest, SCHEMA_VERSION};
use serde_json::json;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn catalog() -> Vec<AgentProbe> {
    [
        ("codex", "Codex", "codex"),
        ("claude-code", "Claude Code", "claude"),
        ("opencode", "OpenCode", "opencode"),
        ("openclaw", "OpenClaw", "openclaw"),
        ("reasonix", "Reasonix", "reasonix"),
    ]
    .into_iter()
    .map(|(id, name, command)| AgentProbe {
        id: id.into(),
        name: name.into(),
        commands: vec![command.into()],
    })
    .collect()
}

fn manifest() -> Manifest {
    Manifest {
        schema_version: SCHEMA_VERSION,
        store_id: "test".into(),
        revision: 7,
        writers: vec!["codex".into(), "claude-code".into()],
        readers: vec!["opencode".into(), "openclaw".into(), "private-agent".into()],
        updated_by: None,
        updated_at: None,
    }
}

fn managed_rules(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        "before\n<!-- BEGIN MOMONOGI -->\nmanaged\n<!-- END MOMONOGI -->\n",
    )
    .unwrap();
}

fn managed_hooks(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut hooks = serde_json::Map::new();
    for event in ["SessionStart", "UserPromptSubmit", "PreCompact"] {
        hooks.insert(
            event.into(),
            json!([{"hooks": [{
                "type": "command",
                "command": "momo hook",
                "statusMessage": format!("Momonogi: {event} continuity (managed v1)")
            }]}]),
        );
    }
    fs::write(path, serde_json::to_vec(&json!({"hooks": hooks})).unwrap()).unwrap();
}

fn executable(path: &Path) {
    fs::write(path, "#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn find<'a>(
    agents: &'a [momonogi::discovery::DiscoveredAgent],
    id: &str,
) -> &'a momonogi::discovery::DiscoveredAgent {
    agents.iter().find(|agent| agent.id == id).unwrap()
}

#[test]
fn discovers_four_known_hosts_from_temporary_layouts() {
    let base = TempDir::new().unwrap();
    let home = base.path().join("home");
    let bin = base.path().join("bin");
    let codex_project = base.path().join("codex-project");
    let openclaw_workspace = base.path().join("openclaw-workspace");
    fs::create_dir_all(&bin).unwrap();
    for command in ["codex", "claude", "opencode", "openclaw"] {
        executable(&bin.join(command));
    }

    managed_rules(&home.join(".codex/AGENTS.md"));
    managed_rules(&home.join(".claude/CLAUDE.md"));
    fs::create_dir_all(home.join(".config/opencode")).unwrap();
    fs::write(home.join(".config/opencode/AGENTS.md"), "user rules\n").unwrap();
    managed_rules(&openclaw_workspace.join("AGENTS.md"));
    managed_hooks(&codex_project.join(".codex/hooks.json"));
    managed_hooks(&home.join(".claude/settings.json"));

    let manifest = manifest();
    let catalog = catalog();
    let path = OsString::from(bin.as_os_str());
    let agents = discover_agents(DiscoveryInput {
        home: &home,
        path: &path,
        manifest: Some(&manifest),
        catalog: &catalog,
        openclaw_workspaces: &[openclaw_workspace],
        codex_projects: &[codex_project],
    });

    let codex = find(&agents, "codex");
    assert!(codex.installed && codex.configured && codex.managed);
    assert_eq!(codex.role, AgentRole::Writer);
    assert_eq!(codex.hook_state, ManagedHookState::Active);

    let claude = find(&agents, "claude-code");
    assert!(claude.installed && claude.configured && claude.managed);
    assert_eq!(claude.role, AgentRole::Writer);
    assert_eq!(claude.hook_state, ManagedHookState::Active);

    let opencode = find(&agents, "opencode");
    assert!(opencode.installed && opencode.configured && !opencode.managed);
    assert_eq!(opencode.role, AgentRole::Reader);
    assert_eq!(opencode.hook_state, ManagedHookState::NotApplicable);

    let openclaw = find(&agents, "openclaw");
    assert!(openclaw.installed && openclaw.configured && openclaw.managed);
    assert_eq!(openclaw.role, AgentRole::Reader);
    assert_eq!(openclaw.hook_state, ManagedHookState::NotApplicable);

    let private = find(&agents, "private-agent");
    assert_eq!(private.name, "private-agent");
    assert_eq!(private.role, AgentRole::Reader);
    assert!(!private.installed && !private.configured);
    assert!(agents.iter().all(|agent| agent.id != "reasonix"));
}

#[test]
fn reports_partial_and_invalid_managed_configuration() {
    let base = TempDir::new().unwrap();
    let home = base.path().join("home");
    let bin = base.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    executable(&bin.join("claude"));
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::write(
        home.join(".claude/CLAUDE.md"),
        "<!-- BEGIN MOMONOGI -->\nbroken\n",
    )
    .unwrap();
    fs::write(home.join(".claude/settings.json"), "not json").unwrap();

    let catalog = catalog();
    let path = OsString::from(bin.as_os_str());
    let agents = discover_agents(DiscoveryInput {
        home: &home,
        path: &path,
        manifest: None,
        catalog: &catalog,
        openclaw_workspaces: &[],
        codex_projects: &[],
    });
    let claude = find(&agents, "claude-code");
    assert!(claude.configured && !claude.managed);
    assert_eq!(claude.hook_state, ManagedHookState::Invalid);
    assert!(claude.config_issue.is_some());
}

#[test]
fn ignores_non_executable_path_entries() {
    let base = TempDir::new().unwrap();
    let bin = base.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("reasonix"), "not executable\n").unwrap();
    let catalog = catalog();
    let path = OsString::from(bin.as_os_str());
    let agents = discover_agents(DiscoveryInput {
        home: base.path(),
        path: &path,
        manifest: None,
        catalog: &catalog,
        openclaw_workspaces: &[],
        codex_projects: &[],
    });
    assert!(agents.is_empty());
}
