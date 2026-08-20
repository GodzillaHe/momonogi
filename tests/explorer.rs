use momonogi::explorer::{ArchiveFilter, MemoryFilter, StoreSource, index_memories, read_memory};
use momonogi::registry::StoreKind;
use momonogi::store::{Manifest, SCHEMA_VERSION, write_manifest};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn store(path: &Path, id: &str) {
    fs::create_dir_all(path).unwrap();
    write_manifest(
        path,
        &Manifest {
            schema_version: SCHEMA_VERSION,
            store_id: id.into(),
            revision: 1,
            writers: vec!["codex".into()],
            readers: Vec::new(),
            updated_by: None,
            updated_at: None,
        },
    )
    .unwrap();
}

fn note(path: &Path, name: &str, kind: &str, scope: &str, tags: &str, body: &str) {
    fs::write(
        path,
        format!(
            "---\nname: {name}\ndescription: {name} description\ntype: {kind}\nscope: {scope}\nstatus: active\ncreated: 2026-08-20\nupdated: 2026-08-20\nrevision: 2\ntags: {tags}\n---\n\n{body}\n"
        ),
    )
    .unwrap();
}

fn fixtures() -> (TempDir, Vec<StoreSource>) {
    let base = TempDir::new().unwrap();
    let global = base.path().join("global");
    let project = base.path().join("project");
    store(&global, "global");
    store(&project, "atlas");
    note(
        &global.join("preference.md"),
        "Preference",
        "user",
        "global",
        "workflow, agents",
        "Complete global body.",
    );
    note(
        &project.join("decision.md"),
        "Decision",
        "reference",
        "repo",
        "architecture",
        "Project detail.",
    );
    fs::create_dir_all(project.join("archive")).unwrap();
    note(
        &project.join("archive/old.md"),
        "Old note",
        "user",
        "repo",
        "legacy",
        "Archived body.",
    );
    let sources = vec![
        StoreSource {
            kind: StoreKind::Global,
            path: global,
        },
        StoreSource {
            kind: StoreKind::Project,
            path: project,
        },
    ];
    (base, sources)
}

#[test]
fn searches_metadata_and_tags_across_global_and_project_stores() {
    let (_base, sources) = fixtures();
    let by_tag = index_memories(
        &sources,
        &MemoryFilter {
            search: "architecture".into(),
            ..MemoryFilter::default()
        },
    );
    assert_eq!(by_tag.notes.len(), 1);
    assert_eq!(by_tag.notes[0].store_id, "atlas");

    let archived = index_memories(
        &sources,
        &MemoryFilter {
            archive: ArchiveFilter::Archived,
            statuses: vec!["archived".into()],
            scopes: vec!["repo".into()],
            memory_types: vec!["user".into()],
            ..MemoryFilter::default()
        },
    );
    assert_eq!(archived.notes.len(), 1);
    assert!(archived.notes[0].archived);
    assert_eq!(archived.notes[0].slug, "old.md");
}

#[test]
fn keeps_valid_notes_when_another_note_is_malformed() {
    let (_base, sources) = fixtures();
    fs::write(sources[0].path.join("broken.md"), "not frontmatter\n").unwrap();

    let index = index_memories(&sources, &MemoryFilter::default());
    assert_eq!(index.notes.len(), 3);
    assert_eq!(index.issues.len(), 1);
    assert_eq!(index.issues[0].slug.as_deref(), Some("broken.md"));
}

#[test]
fn isolates_notes_with_invalid_tags() {
    let (_base, sources) = fixtures();
    note(
        &sources[0].path.join("invalid-tag.md"),
        "Invalid tag",
        "user",
        "global",
        "bad/tag",
        "Body.",
    );

    let index = index_memories(&sources, &MemoryFilter::default());
    assert_eq!(index.notes.len(), 3);
    assert_eq!(index.issues.len(), 1);
    assert_eq!(index.issues[0].slug.as_deref(), Some("invalid-tag.md"));
    assert!(index.issues[0].message.contains("invalid tag"));
}

#[test]
fn reads_complete_active_and_archived_note_details() {
    let (_base, sources) = fixtures();
    let active = read_memory(&sources[0], "preference.md", false).unwrap();
    assert!(active.body.contains("Complete global body."));
    assert!(active.content.starts_with("---\n"));

    let archived = read_memory(&sources[1], "old.md", true).unwrap();
    assert!(archived.summary.archived);
    assert!(archived.body.contains("Archived body."));
}
