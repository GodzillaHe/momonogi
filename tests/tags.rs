use momonogi::store::{Manifest, SCHEMA_VERSION, load_manifest, sha, write_manifest};
use momonogi::tag::{TagAction, change_tag, list_tags, normalize_tag, tags_from_fields};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn momo() -> Command {
    Command::cargo_bin("momo").expect("momo binary")
}

fn fixture() -> (TempDir, std::path::PathBuf) {
    let base = TempDir::new().unwrap();
    let root = base.path().join("store");
    fs::create_dir_all(&root).unwrap();
    write_manifest(
        &root,
        &Manifest {
            schema_version: SCHEMA_VERSION,
            store_id: "test".into(),
            revision: 1,
            writers: vec!["codex".into()],
            readers: vec!["opencode".into()],
            updated_by: None,
            updated_at: None,
        },
    )
    .unwrap();
    fs::write(
        root.join("note.md"),
        "---\nname: Note\ndescription: Note description\ntype: user\nscope: global\nstatus: active\ncreated: 2026-08-20\nupdated: 2026-08-20\nrevision: 1\ntags: Workflow, agents, workflow\n---\n\nBody.\n",
    )
    .unwrap();
    fs::write(root.join("MEMORY.md"), "stale\n").unwrap();
    (base, root)
}

fn etag(root: &Path) -> String {
    sha(fs::read(root.join("note.md")).unwrap())
}

#[test]
fn normalizes_and_deduplicates_tags() {
    assert_eq!(normalize_tag("  Design System  ").unwrap(), "design-system");
    assert!(normalize_tag("bad/tag").is_err());
    let (_base, root) = fixture();
    let note = momonogi::store::notes(&root)
        .unwrap()
        .remove("note.md")
        .unwrap();
    assert_eq!(
        tags_from_fields(&note.fields).unwrap(),
        vec!["agents", "workflow"]
    );
    assert_eq!(list_tags(&root).unwrap()[0].name, "agents");
}

#[test]
fn mutations_require_writer_and_current_etag_and_update_index() {
    let (_base, root) = fixture();
    let current = etag(&root);
    assert!(
        change_tag(
            &root,
            "note.md",
            "design",
            TagAction::Add,
            "opencode",
            &current,
        )
        .unwrap_err()
        .to_string()
        .contains("not a configured writer")
    );
    assert!(
        change_tag(&root, "note.md", "design", TagAction::Add, "codex", "stale",)
            .unwrap_err()
            .to_string()
            .contains("etag conflict")
    );

    let added = change_tag(
        &root,
        "note.md",
        "Design",
        TagAction::Add,
        "codex",
        &current,
    )
    .unwrap();
    assert!(added.changed);
    assert_eq!(added.revision, 2);
    assert_eq!(added.tags, vec!["agents", "design", "workflow"]);
    assert!(
        fs::read_to_string(root.join("MEMORY.md"))
            .unwrap()
            .contains("note.md")
    );
    assert_eq!(load_manifest(&root).unwrap().manifest.revision, 1);

    let removed = change_tag(
        &root,
        "note.md",
        "workflow",
        TagAction::Remove,
        "codex",
        &added.etag,
    )
    .unwrap();
    assert_eq!(removed.revision, 3);
    assert_eq!(removed.tags, vec!["agents", "design"]);
}

#[test]
fn duplicate_and_missing_mutations_are_noops() {
    let (_base, root) = fixture();
    let current = etag(&root);
    let duplicate = change_tag(
        &root,
        "note.md",
        "WORKFLOW",
        TagAction::Add,
        "codex",
        &current,
    )
    .unwrap();
    assert!(!duplicate.changed);
    assert_eq!(duplicate.revision, 1);
    assert_eq!(duplicate.etag, current);

    let missing = change_tag(
        &root,
        "note.md",
        "absent",
        TagAction::Remove,
        "codex",
        &current,
    )
    .unwrap();
    assert!(!missing.changed);
    assert_eq!(missing.etag, current);
}

#[test]
fn cli_lists_adds_and_removes_tags() {
    let (_base, root) = fixture();
    let listed = momo()
        .args(["tag", "list", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(listed.status.success());
    let listed: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed[0]["name"], "agents");

    let added = momo()
        .args([
            "tag",
            "add",
            root.to_str().unwrap(),
            "note.md",
            "Design System",
            "--agent",
            "codex",
            "--if-match",
            &etag(&root),
        ])
        .output()
        .unwrap();
    assert!(added.status.success());
    let added: serde_json::Value = serde_json::from_slice(&added.stdout).unwrap();
    assert_eq!(added["tag"], "design-system");

    momo()
        .args([
            "tag",
            "remove",
            root.to_str().unwrap(),
            "note.md",
            "design-system",
            "--agent",
            "codex",
            "--if-match",
            added["etag"].as_str().unwrap(),
        ])
        .assert()
        .success();
}
use assert_cmd::Command;
