use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::process::Stdio;
use tempfile::TempDir;

fn momo() -> Command {
    Command::cargo_bin("momo").expect("momo binary")
}

fn init(base: &TempDir) -> std::path::PathBuf {
    let root = base.path().join("store");
    momo()
        .args([
            "init",
            root.to_str().unwrap(),
            "--store-id",
            "test",
            "--writer",
            "codex",
            "--writer",
            "claude-code",
            "--reader",
            "opencode",
            "--reader",
            "openclaw",
        ])
        .assert()
        .success();
    root
}

fn note(path: &std::path::Path, name: &str, kind: &str, body: &str) {
    let reflection = if matches!(kind, "feedback" | "project") {
        "\nWhy: durable\n\nHow to apply: reuse it.\n"
    } else {
        ""
    };
    fs::write(path, format!("---\nname: {name}\ndescription: {name} description\ntype: {kind}\ncreated: 2026-08-20\nupdated: 2026-08-20\n---\n\n{body}{reflection}")).unwrap();
}

fn put(root: &std::path::Path, source: &std::path::Path, agent: &str) -> Value {
    let output = momo()
        .args([
            "put",
            root.to_str().unwrap(),
            source.to_str().unwrap(),
            "--agent",
            agent,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn access(root: &std::path::Path) -> Value {
    let output = momo()
        .args(["access", "list", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn grant(root: &std::path::Path, agent: &str, role: &str, actor: &str, etag: &str) -> Value {
    let output = momo()
        .args([
            "access",
            "grant",
            root.to_str().unwrap(),
            agent,
            "--role",
            role,
            "--by",
            actor,
            "--if-match",
            etag,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
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
fn access_list_and_mutations_are_revisioned_and_role_aware() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let initial = access(&root);
    assert_eq!(initial["revision"], 1);
    assert_eq!(
        initial["writers"],
        serde_json::json!(["claude-code", "codex"])
    );
    assert_eq!(
        initial["readers"],
        serde_json::json!(["openclaw", "opencode"])
    );
    assert_eq!(initial["etag"].as_str().unwrap().len(), 64);

    let promoted = grant(
        &root,
        "opencode",
        "writer",
        "codex",
        initial["etag"].as_str().unwrap(),
    );
    assert_eq!(promoted["changed"], true);
    assert_eq!(promoted["revision"], 2);
    assert!(
        promoted["writers"]
            .as_array()
            .unwrap()
            .contains(&Value::from("opencode"))
    );

    let source = base.path().join("opencode.md");
    note(&source, "OpenCode", "user", "writer now\n");
    assert_eq!(put(&root, &source, "opencode")["revision"], 1);

    let downgraded = grant(
        &root,
        "claude-code",
        "reader",
        "opencode",
        promoted["etag"].as_str().unwrap(),
    );
    assert_eq!(downgraded["revision"], 3);
    let denied = base.path().join("denied-after-downgrade.md");
    note(&denied, "Denied", "user", "reader now\n");
    momo()
        .args([
            "put",
            root.to_str().unwrap(),
            denied.to_str().unwrap(),
            "--agent",
            "claude-code",
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("not a configured writer"));

    let revoked = momo()
        .args([
            "access",
            "revoke",
            root.to_str().unwrap(),
            "openclaw",
            "--by",
            "codex",
            "--if-match",
            downgraded["etag"].as_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(revoked.status.success());
    let revoked: Value = serde_json::from_slice(&revoked.stdout).unwrap();
    assert_eq!(revoked["revision"], 4);
    assert!(
        !revoked["readers"]
            .as_array()
            .unwrap()
            .contains(&Value::from("openclaw"))
    );

    let arbitrary = grant(
        &root,
        "gemini-cli",
        "reader",
        "codex",
        revoked["etag"].as_str().unwrap(),
    );
    assert!(
        arbitrary["readers"]
            .as_array()
            .unwrap()
            .contains(&Value::from("gemini-cli"))
    );
}

#[test]
fn access_rejects_unauthorized_stale_and_last_writer_changes() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let initial = access(&root);
    let etag = initial["etag"].as_str().unwrap();

    momo()
        .args([
            "access",
            "grant",
            root.to_str().unwrap(),
            "opencode",
            "--role",
            "writer",
            "--by",
            "openclaw",
            "--if-match",
            etag,
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("not a configured writer"));

    let changed = grant(&root, "claude-code", "reader", "codex", etag);
    momo()
        .args([
            "access",
            "grant",
            root.to_str().unwrap(),
            "opencode",
            "--role",
            "writer",
            "--by",
            "codex",
            "--if-match",
            etag,
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("manifest etag conflict"));

    momo()
        .args([
            "access",
            "set",
            root.to_str().unwrap(),
            "codex",
            "--role",
            "reader",
            "--by",
            "codex",
            "--if-match",
            changed["etag"].as_str().unwrap(),
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("final writer"));
    momo()
        .args([
            "access",
            "revoke",
            root.to_str().unwrap(),
            "codex",
            "--by",
            "codex",
            "--if-match",
            changed["etag"].as_str().unwrap(),
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("final writer"));
}

#[test]
fn access_noop_preserves_manifest_etag_and_revision() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let initial = access(&root);
    let before = fs::read(root.join(".momonogi.json")).unwrap();
    let unchanged = grant(
        &root,
        "opencode",
        "reader",
        "codex",
        initial["etag"].as_str().unwrap(),
    );
    assert_eq!(unchanged["changed"], false);
    assert_eq!(unchanged["etag"], initial["etag"]);
    assert_eq!(unchanged["revision"], initial["revision"]);
    assert_eq!(fs::read(root.join(".momonogi.json")).unwrap(), before);
}

#[test]
fn legacy_schema_one_manifest_defaults_to_revision_one() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    fs::write(
        root.join(".momonogi.json"),
        r#"{
  "schema_version": 1,
  "store_id": "legacy",
  "writers": ["codex"],
  "readers": ["opencode"]
}
"#,
    )
    .unwrap();
    let legacy = access(&root);
    assert_eq!(legacy["revision"], 1);
    let updated = grant(
        &root,
        "opencode",
        "writer",
        "codex",
        legacy["etag"].as_str().unwrap(),
    );
    assert_eq!(updated["revision"], 2);
    assert_eq!(updated["updated_by"], "codex");
}

#[test]
fn two_writers_share_revisioned_store_and_readers_cannot_write() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let one = base.path().join("one.md");
    let two = base.path().join("two.md");
    let denied = base.path().join("denied.md");
    note(&one, "One", "user", "one\n");
    note(&two, "Two", "feedback", "two\n");
    note(&denied, "Denied", "user", "no\n");
    assert_eq!(put(&root, &one, "codex")["revision"], 1);
    assert_eq!(put(&root, &two, "claude-code")["revision"], 1);
    momo()
        .args([
            "put",
            root.to_str().unwrap(),
            denied.to_str().unwrap(),
            "--agent",
            "opencode",
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("not a configured writer"));
    let index = fs::read_to_string(root.join("MEMORY.md")).unwrap();
    assert!(index.contains("one.md"));
    assert!(index.contains("two.md"));
    assert!(!index.contains("denied.md"));
}

#[test]
fn stale_etag_is_rejected_and_current_etag_updates_revision() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let source = base.path().join("preference.md");
    note(&source, "Memory", "user", "first\n");
    let created = put(&root, &source, "codex");
    note(&source, "Memory", "user", "second\n");
    momo()
        .args([
            "put",
            root.to_str().unwrap(),
            source.to_str().unwrap(),
            "--agent",
            "claude-code",
            "--if-match",
            "bad",
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("etag conflict"));
    let output = momo()
        .args([
            "put",
            root.to_str().unwrap(),
            source.to_str().unwrap(),
            "--agent",
            "claude-code",
            "--if-match",
            created["etag"].as_str().unwrap(),
        ])
        .output()
        .unwrap();
    let updated: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(updated["revision"], 2);
    assert!(
        fs::read_to_string(root.join("preference.md"))
            .unwrap()
            .contains("updated_by: claude-code")
    );
}

#[test]
fn list_json_is_metadata_only_and_archive_is_visible_on_request() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let source = base.path().join("preference.md");
    note(&source, "Memory", "user", "private body\n");
    let created = put(&root, &source, "codex");
    let output = momo()
        .args(["list", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let rows: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("private body"));
    assert!(!output.stdout.contains(&0x1b));
    momo()
        .args([
            "archive",
            root.to_str().unwrap(),
            "preference.md",
            "--agent",
            "claude-code",
            "--if-match",
            created["etag"].as_str().unwrap(),
        ])
        .assert()
        .success();
    let output = momo()
        .args([
            "list",
            root.to_str().unwrap(),
            "--include-archived",
            "--json",
        ])
        .output()
        .unwrap();
    let rows: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(rows[0]["archived"], true);
    assert_eq!(rows[0]["status"], "archived");
    assert_eq!(rows[0]["revision"], 2);

    let created = put(&root, &source, "codex");
    momo()
        .args([
            "archive",
            root.to_str().unwrap(),
            "preference.md",
            "--agent",
            "claude-code",
            "--if-match",
            created["etag"].as_str().unwrap(),
        ])
        .assert()
        .success();
    let output = momo()
        .args([
            "list",
            root.to_str().unwrap(),
            "--include-archived",
            "--json",
        ])
        .output()
        .unwrap();
    let rows: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 2);
    assert_ne!(rows[0]["slug"], rows[1]["slug"]);
}

#[test]
fn concurrent_processes_do_not_lose_writes() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let one = base.path().join("one.md");
    let two = base.path().join("two.md");
    note(&one, "One", "user", "one\n");
    note(&two, "Two", "user", "two\n");
    let binary = assert_cmd::cargo::cargo_bin("momo");
    let mut first = std::process::Command::new(&binary)
        .args([
            "put",
            root.to_str().unwrap(),
            one.to_str().unwrap(),
            "--agent",
            "codex",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut second = std::process::Command::new(&binary)
        .args([
            "put",
            root.to_str().unwrap(),
            two.to_str().unwrap(),
            "--agent",
            "claude-code",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    let index = fs::read_to_string(root.join("MEMORY.md")).unwrap();
    assert!(index.contains("one.md"));
    assert!(index.contains("two.md"));
}

#[test]
fn doctor_and_reindex_detect_and_repair_stale_index() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let source = base.path().join("repair.md");
    note(&source, "Memory", "user", "body\n");
    put(&root, &source, "codex");
    fs::write(root.join("MEMORY.md"), "stale\n").unwrap();
    momo()
        .args(["doctor", root.to_str().unwrap()])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("does not match"));
    momo()
        .args(["reindex", root.to_str().unwrap(), "--agent", "claude-code"])
        .assert()
        .success();
    momo()
        .args(["doctor", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("doctor: clean"));
}

#[test]
fn index_name_is_reserved_case_insensitively() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let source = base.path().join("memory.md");
    note(&source, "Reserved", "user", "body\n");
    momo()
        .args([
            "put",
            root.to_str().unwrap(),
            source.to_str().unwrap(),
            "--agent",
            "codex",
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("cannot be memory.md"));
    assert!(root.join("MEMORY.md").is_file());
}

#[test]
fn configure_is_idempotent_and_merges_lifecycle_hooks() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let home = base.path().join("home");
    let project = base.path().join("project");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::write(
        home.join(".claude/settings.json"),
        r#"{"model":"keep-me","hooks":{"Stop":[{"hooks":[{"type":"command","command":"audit"}]}]}}"#,
    )
    .unwrap();
    for _ in 0..2 {
        momo()
            .env("HOME", &home)
            .args([
                "configure",
                "--host",
                "codex",
                "--host",
                "claude",
                "--host",
                "opencode",
                "--host",
                "openclaw",
                "--codex-project",
                project.to_str().unwrap(),
                "--openclaw-workspace",
                project.to_str().unwrap(),
                "--memory-root",
                root.to_str().unwrap(),
            ])
            .assert()
            .success();
    }
    for path in [
        home.join(".codex/AGENTS.md"),
        home.join(".claude/CLAUDE.md"),
        home.join(".config/opencode/AGENTS.md"),
    ] {
        let text = fs::read_to_string(path).unwrap();
        assert_eq!(text.matches("BEGIN MOMONOGI").count(), 1);
        assert!(text.contains("only when continuity"));
    }
    for path in [
        home.join(".claude/settings.json"),
        project.join(".codex/hooks.json"),
    ] {
        let config: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        for event in ["SessionStart", "UserPromptSubmit", "PreCompact"] {
            let managed = config["hooks"][event]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|group| group["hooks"].as_array().unwrap())
                .filter(|handler| {
                    handler["statusMessage"]
                        .as_str()
                        .is_some_and(|value| value.starts_with("Momonogi:"))
                })
                .count();
            assert_eq!(managed, 1);
        }
    }
    let claude: Value =
        serde_json::from_str(&fs::read_to_string(home.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(claude["model"], "keep-me");
    assert_eq!(claude["hooks"]["Stop"][0]["hooks"][0]["command"], "audit");
    let shared = fs::read_to_string(project.join("AGENTS.md")).unwrap();
    assert_eq!(shared.matches("BEGIN MOMONOGI").count(), 1);
    assert!(shared.contains("If the current host is not OpenClaw"));
    assert!(shared.contains("does not change its Momonogi role"));
    assert!(shared.contains("OpenClaw's native memory as primary"));
    assert!(!shared.contains("`OpenClaw` is a read-only consumer"));
}

#[test]
fn configure_follows_manifest_roles_and_removes_only_managed_hooks() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let home = base.path().join("home");
    let project = base.path().join("project");
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::create_dir_all(project.join(".codex")).unwrap();
    fs::write(
        home.join(".claude/settings.json"),
        r#"{"model":"keep","hooks":{"Stop":[{"hooks":[{"type":"command","command":"claude-audit"}]}]}}"#,
    )
    .unwrap();
    fs::write(
        project.join(".codex/hooks.json"),
        r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"codex-audit"}]}]}}"#,
    )
    .unwrap();

    momo()
        .env("HOME", &home)
        .args([
            "configure",
            "--host",
            "codex",
            "--host",
            "claude",
            "--codex-project",
            project.to_str().unwrap(),
            "--memory-root",
            root.to_str().unwrap(),
        ])
        .assert()
        .success();
    let claude: Value =
        serde_json::from_str(&fs::read_to_string(home.join(".claude/settings.json")).unwrap())
            .unwrap();
    let codex: Value =
        serde_json::from_str(&fs::read_to_string(project.join(".codex/hooks.json")).unwrap())
            .unwrap();
    assert_eq!(managed_hook_count(&claude), 3);
    assert_eq!(managed_hook_count(&codex), 3);

    let current = access(&root);
    let current = grant(
        &root,
        "claude-code",
        "reader",
        "codex",
        current["etag"].as_str().unwrap(),
    );
    momo()
        .env("HOME", &home)
        .args([
            "configure",
            "--host",
            "claude",
            "--memory-root",
            root.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(
        fs::read_to_string(home.join(".claude/CLAUDE.md"))
            .unwrap()
            .contains("Read Only")
    );
    let claude: Value =
        serde_json::from_str(&fs::read_to_string(home.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(managed_hook_count(&claude), 0);
    assert_eq!(claude["model"], "keep");
    assert_eq!(
        claude["hooks"]["Stop"][0]["hooks"][0]["command"],
        "claude-audit"
    );

    let current = grant(
        &root,
        "opencode",
        "writer",
        "codex",
        current["etag"].as_str().unwrap(),
    );
    grant(
        &root,
        "codex",
        "reader",
        "opencode",
        current["etag"].as_str().unwrap(),
    );
    momo()
        .env("HOME", &home)
        .args([
            "configure",
            "--host",
            "codex",
            "--host",
            "opencode",
            "--codex-project",
            project.to_str().unwrap(),
            "--memory-root",
            root.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(
        fs::read_to_string(home.join(".codex/AGENTS.md"))
            .unwrap()
            .contains("Read Only")
    );
    assert!(
        fs::read_to_string(home.join(".config/opencode/AGENTS.md"))
            .unwrap()
            .contains("equal trusted writer with agent id `opencode`")
    );
    let codex: Value =
        serde_json::from_str(&fs::read_to_string(project.join(".codex/hooks.json")).unwrap())
            .unwrap();
    assert_eq!(managed_hook_count(&codex), 0);
    assert_eq!(
        codex["hooks"]["Stop"][0]["hooks"][0]["command"],
        "codex-audit"
    );
}

#[test]
fn openclaw_role_changes_remain_host_conditional_in_shared_projects() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let home = base.path().join("home");
    let project = base.path().join("project");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    let current = access(&root);
    let current = grant(
        &root,
        "openclaw",
        "writer",
        "codex",
        current["etag"].as_str().unwrap(),
    );
    momo()
        .env("HOME", &home)
        .args([
            "configure",
            "--host",
            "openclaw",
            "--openclaw-workspace",
            project.to_str().unwrap(),
            "--memory-root",
            root.to_str().unwrap(),
        ])
        .assert()
        .success();
    let shared = fs::read_to_string(project.join("AGENTS.md")).unwrap();
    assert!(shared.contains("Host Conditional"));
    assert!(shared.contains("If the current host is not OpenClaw"));
    assert!(shared.contains("equal trusted writer with agent id `openclaw`"));

    momo()
        .args([
            "access",
            "revoke",
            root.to_str().unwrap(),
            "openclaw",
            "--by",
            "codex",
            "--if-match",
            current["etag"].as_str().unwrap(),
        ])
        .assert()
        .success();
    momo()
        .env("HOME", &home)
        .args([
            "configure",
            "--host",
            "openclaw",
            "--openclaw-workspace",
            project.to_str().unwrap(),
            "--memory-root",
            root.to_str().unwrap(),
        ])
        .assert()
        .success();
    let shared = fs::read_to_string(project.join("AGENTS.md")).unwrap();
    assert_eq!(shared.matches("BEGIN MOMONOGI").count(), 1);
    assert!(shared.contains("not granted access"));
    assert!(shared.contains("does not change its Momonogi role"));
}

#[test]
fn lifecycle_tracks_dirty_compaction_and_acknowledgement() {
    let base = TempDir::new().unwrap();
    let root = init(&base);
    let hook = |event: &str| -> Value {
        let output = momo()
            .args(["hook", "--memory-root", root.to_str().unwrap()])
            .write_stdin(event)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    };
    let start = hook(r#"{"hook_event_name":"SessionStart","session_id":"s1"}"#);
    assert!(
        start["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .contains("Momonogi")
    );
    hook(r#"{"hook_event_name":"UserPromptSubmit","session_id":"s1","prompt":"work"}"#);
    let compact = hook(r#"{"hook_event_name":"PreCompact","session_id":"s1","trigger":"manual"}"#);
    assert_eq!(compact["continue"], false);
    momo()
        .args(["sync", "mark", root.to_str().unwrap(), "--session-id", "s1"])
        .assert()
        .success();
    let compact = hook(r#"{"hook_event_name":"PreCompact","session_id":"s1","trigger":"manual"}"#);
    assert_eq!(compact["continue"], true);
}
