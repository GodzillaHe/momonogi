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
                "--codex-project",
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
