use crate::store::{self, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const REGISTRY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Registry {
    pub schema_version: u32,
    #[serde(default)]
    pub projects: Vec<String>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            schema_version: REGISTRY_SCHEMA_VERSION,
            projects: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreKind {
    Global,
    Project,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreHealth {
    Ready,
    Missing,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreEntry {
    pub kind: StoreKind,
    pub path: String,
    pub health: StoreHealth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
}

fn validate_registry(registry: &Registry) -> Result<()> {
    if registry.schema_version != REGISTRY_SCHEMA_VERSION {
        return Err(Error("unsupported registry schema".into()));
    }
    let mut seen = HashSet::new();
    for project in &registry.projects {
        if project.trim().is_empty() {
            return Err(Error("registry project path cannot be empty".into()));
        }
        if !seen.insert(project) {
            return Err(Error(format!(
                "project store appears more than once in registry: {project}"
            )));
        }
    }
    Ok(())
}

pub fn load_registry(path: &Path) -> Result<Registry> {
    if !path.is_file() {
        return Ok(Registry::default());
    }
    let registry: Registry = serde_json::from_slice(&fs::read(path)?)
        .map_err(|error| Error(format!("cannot read store registry: {error}")))?;
    validate_registry(&registry)?;
    Ok(registry)
}

pub fn write_registry(path: &Path, registry: &Registry) -> Result<()> {
    validate_registry(registry)?;
    let mut data = serde_json::to_vec_pretty(registry)?;
    data.push(b'\n');
    store::atomic_write(path, &data, 0o600)
}

fn canonical_store(path: impl AsRef<Path>) -> Result<PathBuf> {
    let expanded = store::expand_path(path);
    expanded
        .canonicalize()
        .map_err(|error| Error(format!("cannot open store {}: {error}", expanded.display())))
}

pub fn register_project_store(
    registry_path: &Path,
    global_root: &Path,
    project_root: &Path,
) -> Result<Registry> {
    let global_root = canonical_store(global_root)?;
    let project_root = canonical_store(project_root)?;
    if project_root == global_root {
        return Err(Error(
            "global store cannot be registered as a project".into(),
        ));
    }
    store::load_manifest(&project_root)
        .map_err(|error| Error(format!("invalid project store: {error}")))?;

    let project = project_root.to_string_lossy().into_owned();
    let mut registry = load_registry(registry_path)?;
    if registry.projects.iter().any(|current| current == &project) {
        return Err(Error(format!(
            "project store is already registered: {project}"
        )));
    }
    registry.projects.push(project);
    registry.projects.sort();
    write_registry(registry_path, &registry)?;
    Ok(registry)
}

pub fn remove_project_store(registry_path: &Path, project_root: &Path) -> Result<Registry> {
    let requested = store::expand_path(project_root);
    let requested = requested.canonicalize().unwrap_or(requested);
    let mut registry = load_registry(registry_path)?;
    let before = registry.projects.len();
    registry
        .projects
        .retain(|current| Path::new(current) != requested);
    if registry.projects.len() != before {
        write_registry(registry_path, &registry)?;
    }
    Ok(registry)
}

fn inspect_store(kind: StoreKind, path: PathBuf) -> StoreEntry {
    let path_text = path.to_string_lossy().into_owned();
    if !path.is_dir() {
        return StoreEntry {
            kind,
            path: path_text,
            health: StoreHealth::Missing,
            store_id: None,
            revision: None,
            issue: Some("store directory is missing".into()),
        };
    }
    match store::load_manifest(&path) {
        Ok(loaded) => StoreEntry {
            kind,
            path: path_text,
            health: StoreHealth::Ready,
            store_id: Some(loaded.manifest.store_id),
            revision: Some(loaded.manifest.revision),
            issue: None,
        },
        Err(error) => StoreEntry {
            kind,
            path: path_text,
            health: StoreHealth::Invalid,
            store_id: None,
            revision: None,
            issue: Some(error.to_string()),
        },
    }
}

pub fn inspect_registry(registry_path: &Path, global_root: &Path) -> Result<Vec<StoreEntry>> {
    let registry = load_registry(registry_path)?;
    let mut stores = Vec::with_capacity(registry.projects.len() + 1);
    stores.push(inspect_store(
        StoreKind::Global,
        store::expand_path(global_root),
    ));
    stores.extend(
        registry
            .projects
            .into_iter()
            .map(PathBuf::from)
            .map(|path| inspect_store(StoreKind::Project, path)),
    );
    Ok(stores)
}
