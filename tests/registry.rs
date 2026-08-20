use momonogi::registry::{
    REGISTRY_SCHEMA_VERSION, Registry, StoreHealth, StoreKind, inspect_registry,
    register_project_store, remove_project_store, write_registry,
};
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

#[test]
fn persists_global_and_explicit_project_stores() {
    let base = TempDir::new().unwrap();
    let registry = base.path().join("config/stores.json");
    let global = base.path().join("global");
    let project = base.path().join("project");
    store(&global, "global");
    store(&project, "momonogi");

    register_project_store(&registry, &global, &project).unwrap();
    let stores = inspect_registry(&registry, &global).unwrap();
    assert_eq!(stores.len(), 2);
    assert_eq!(stores[0].kind, StoreKind::Global);
    assert_eq!(stores[0].health, StoreHealth::Ready);
    assert_eq!(stores[1].kind, StoreKind::Project);
    assert_eq!(stores[1].store_id.as_deref(), Some("momonogi"));
    assert_eq!(stores[1].revision, Some(1));
}

#[test]
fn rejects_duplicate_missing_and_invalid_project_stores() {
    let base = TempDir::new().unwrap();
    let registry = base.path().join("config/stores.json");
    let global = base.path().join("global");
    let project = base.path().join("project");
    store(&global, "global");
    store(&project, "project");

    register_project_store(&registry, &global, &project).unwrap();
    assert!(
        register_project_store(&registry, &global, &project)
            .unwrap_err()
            .to_string()
            .contains("already registered")
    );
    assert!(
        register_project_store(&registry, &global, &base.path().join("missing"))
            .unwrap_err()
            .to_string()
            .contains("cannot open store")
    );

    let invalid = base.path().join("invalid");
    fs::create_dir_all(&invalid).unwrap();
    assert!(
        register_project_store(&registry, &global, &invalid)
            .unwrap_err()
            .to_string()
            .contains("invalid project store")
    );
}

#[test]
fn reports_missing_registered_paths_and_removes_only_registry_data() {
    let base = TempDir::new().unwrap();
    let registry_path = base.path().join("config/stores.json");
    let global = base.path().join("global");
    let project = base.path().join("project");
    let missing = base.path().join("missing-project");
    store(&global, "global");
    store(&project, "project");
    fs::write(project.join("keep.txt"), "memory data").unwrap();
    write_registry(
        &registry_path,
        &Registry {
            schema_version: REGISTRY_SCHEMA_VERSION,
            projects: vec![missing.to_string_lossy().into_owned()],
        },
    )
    .unwrap();

    let stores = inspect_registry(&registry_path, &global).unwrap();
    assert_eq!(stores[1].health, StoreHealth::Missing);

    register_project_store(&registry_path, &global, &project).unwrap();
    let registry = remove_project_store(&registry_path, &project).unwrap();
    assert!(
        !registry
            .projects
            .iter()
            .any(|path| path == &project.to_string_lossy())
    );
    assert_eq!(
        fs::read_to_string(project.join("keep.txt")).unwrap(),
        "memory data"
    );
}
