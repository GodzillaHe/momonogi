use momonogi::configure::{ConfigurationAction, Host, SyncOptions, apply_host, preview_host};
use momonogi::store::{Manifest, SCHEMA_VERSION, sha, write_manifest};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn manifest(root: &Path, writers: &[&str], readers: &[&str]) {
    fs::create_dir_all(root).unwrap();
    write_manifest(
        root,
        &Manifest {
            schema_version: SCHEMA_VERSION,
            store_id: "test".into(),
            revision: 1,
            writers: writers.iter().map(|value| (*value).into()).collect(),
            readers: readers.iter().map(|value| (*value).into()).collect(),
            updated_by: None,
            updated_at: None,
        },
    )
    .unwrap();
}

fn options<'a>(
    host: Host,
    home: &'a Path,
    memory: &'a Path,
    workspaces: &'a [PathBuf],
    projects: &'a [PathBuf],
    tool: &'a Path,
) -> SyncOptions<'a> {
    SyncOptions {
        host,
        home,
        openclaw_workspaces: workspaces,
        codex_projects: projects,
        memory_root: memory,
        hook_mode: "explicit",
        install_hooks: true,
        tool,
    }
}

fn managed_hook_count(config: &Value) -> usize {
    ["SessionStart", "UserPromptSubmit", "PreCompact"]
        .iter()
        .map(|event| {
            config["hooks"][event]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|group| group["hooks"].as_array())
                .flatten()
                .filter(|handler| {
                    handler["statusMessage"]
                        .as_str()
                        .is_some_and(|value| value.starts_with("Momonogi:"))
                })
                .count()
        })
        .sum()
}

#[test]
fn previews_without_writing_and_reapply_preserves_hashes() {
    let base = TempDir::new().unwrap();
    let memory = base.path().join("memory");
    let home = base.path().join("home");
    let tool = base.path().join("bin/momo");
    manifest(&memory, &["claude-code"], &[]);
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::write(
        home.join(".claude/settings.json"),
        r#"{"model":"keep","hooks":{"Stop":[{"hooks":[{"type":"command","command":"audit"}]}]}}"#,
    )
    .unwrap();
    let workspaces = Vec::new();
    let projects = Vec::new();
    let options = options(Host::Claude, &home, &memory, &workspaces, &projects, &tool);

    let preview = preview_host(&options).unwrap();
    assert_eq!(preview.files.len(), 2);
    assert_eq!(preview.files[0].action, ConfigurationAction::Create);
    assert!(!home.join(".claude/CLAUDE.md").exists());
    assert_eq!(
        managed_hook_count(
            &serde_json::from_str::<Value>(
                &fs::read_to_string(home.join(".claude/settings.json")).unwrap()
            )
            .unwrap()
        ),
        0
    );

    let applied = apply_host(&options, &preview.digest).unwrap();
    assert_eq!(applied.changed_files.len(), 2);
    let rules_hash = sha(fs::read(home.join(".claude/CLAUDE.md")).unwrap());
    let hooks_hash = sha(fs::read(home.join(".claude/settings.json")).unwrap());
    let second = preview_host(&options).unwrap();
    assert!(
        second
            .files
            .iter()
            .all(|file| file.action == ConfigurationAction::Unchanged)
    );
    assert!(
        apply_host(&options, &second.digest)
            .unwrap()
            .changed_files
            .is_empty()
    );
    assert_eq!(
        sha(fs::read(home.join(".claude/CLAUDE.md")).unwrap()),
        rules_hash
    );
    assert_eq!(
        sha(fs::read(home.join(".claude/settings.json")).unwrap()),
        hooks_hash
    );
}

#[test]
fn rejects_stale_preview_before_writing_any_file() {
    let base = TempDir::new().unwrap();
    let memory = base.path().join("memory");
    let home = base.path().join("home");
    let tool = base.path().join("momo");
    manifest(&memory, &["claude-code"], &[]);
    fs::create_dir_all(home.join(".claude")).unwrap();
    let workspaces = Vec::new();
    let projects = Vec::new();
    let options = options(Host::Claude, &home, &memory, &workspaces, &projects, &tool);
    let preview = preview_host(&options).unwrap();
    fs::write(home.join(".claude/CLAUDE.md"), "user change\n").unwrap();

    let error = apply_host(&options, &preview.digest).unwrap_err();
    assert!(error.to_string().contains("preview is stale"));
    assert_eq!(
        fs::read_to_string(home.join(".claude/CLAUDE.md")).unwrap(),
        "user change\n"
    );
    assert!(!home.join(".claude/settings.json").exists());
}

#[test]
fn reader_transition_removes_only_momonogi_hooks() {
    let base = TempDir::new().unwrap();
    let memory = base.path().join("memory");
    let home = base.path().join("home");
    let project = base.path().join("project");
    let tool = base.path().join("momo");
    manifest(&memory, &["codex"], &[]);
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(project.join(".codex")).unwrap();
    fs::write(
        project.join(".codex/hooks.json"),
        r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"keep-audit"}]}]}}"#,
    )
    .unwrap();
    let workspaces = Vec::new();
    let projects = vec![project.clone()];
    let writer = options(Host::Codex, &home, &memory, &workspaces, &projects, &tool);
    let preview = preview_host(&writer).unwrap();
    apply_host(&writer, &preview.digest).unwrap();

    manifest(&memory, &["admin"], &["codex"]);
    let reader = options(Host::Codex, &home, &memory, &workspaces, &projects, &tool);
    let preview = preview_host(&reader).unwrap();
    assert!(preview.files.iter().any(|file| {
        file.path.ends_with("hooks.json") && file.action == ConfigurationAction::RemoveManaged
    }));
    apply_host(&reader, &preview.digest).unwrap();

    let hooks: Value =
        serde_json::from_str(&fs::read_to_string(project.join(".codex/hooks.json")).unwrap())
            .unwrap();
    assert_eq!(managed_hook_count(&hooks), 0);
    assert_eq!(
        hooks["hooks"]["Stop"][0]["hooks"][0]["command"],
        "keep-audit"
    );
    assert!(
        fs::read_to_string(home.join(".codex/AGENTS.md"))
            .unwrap()
            .contains("Read Only")
    );
}

#[test]
fn openclaw_workspace_rules_remain_host_conditional() {
    let base = TempDir::new().unwrap();
    let memory = base.path().join("memory");
    let home = base.path().join("home");
    let workspace = base.path().join("workspace");
    let tool = base.path().join("momo");
    manifest(&memory, &["admin"], &["openclaw"]);
    fs::create_dir_all(&workspace).unwrap();
    fs::write(workspace.join("AGENTS.md"), "shared rules\n").unwrap();
    let workspaces = vec![workspace.clone()];
    let projects = Vec::new();
    let options = options(
        Host::Openclaw,
        &home,
        &memory,
        &workspaces,
        &projects,
        &tool,
    );
    let preview = preview_host(&options).unwrap();
    apply_host(&options, &preview.digest).unwrap();

    let rules = fs::read_to_string(workspace.join("AGENTS.md")).unwrap();
    assert!(rules.starts_with("shared rules\n"));
    assert!(rules.contains("Host Conditional"));
    assert!(rules.contains("If the current host is not OpenClaw"));
    assert!(rules.contains("OpenClaw's native memory as primary"));
}
