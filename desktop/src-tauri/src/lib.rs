use momonogi::discovery::{AgentProbe, DiscoveredAgent, DiscoveryInput};
use serde::Serialize;
use std::ffi::OsString;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapPayload {
    app_version: &'static str,
    core_schema: u32,
    bridge: &'static str,
}

#[tauri::command]
fn bootstrap() -> BootstrapPayload {
    BootstrapPayload {
        app_version: env!("CARGO_PKG_VERSION"),
        core_schema: momonogi::store::SCHEMA_VERSION,
        bridge: "desktop",
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentDiscoveryPayload {
    agents: Vec<DiscoveredAgent>,
    store_root: String,
    store_available: bool,
    store_revision: Option<u64>,
    store_etag: Option<String>,
    store_issue: Option<String>,
}

fn home_directory() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| "cannot determine home directory".to_owned())
}

fn project_workspaces(app: &tauri::AppHandle) -> Result<Vec<PathBuf>, String> {
    let registry = momonogi::registry::load_registry(&registry_path(app)?)
        .map_err(|error| error.to_string())?;
    let mut workspaces = Vec::new();
    for store in registry.projects {
        let store = PathBuf::from(store);
        let workspace = if store.file_name().is_some_and(|name| name == ".momonogi") {
            store
                .parent()
                .map(PathBuf::from)
                .ok_or_else(|| "project store has no parent workspace".to_owned())?
        } else {
            store
        };
        if !workspaces.contains(&workspace) {
            workspaces.push(workspace);
        }
    }
    workspaces.sort();
    Ok(workspaces)
}

fn is_executable(path: &std::path::Path) -> bool {
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

fn momo_executable(home: &std::path::Path, path: &std::ffi::OsStr) -> PathBuf {
    let local = home.join(".local/bin/momo");
    if is_executable(&local) {
        return local;
    }
    std::env::split_paths(path)
        .map(|directory| directory.join("momo"))
        .find(|candidate| is_executable(candidate))
        .unwrap_or_else(|| PathBuf::from("momo"))
}

#[tauri::command]
fn discover_agents(
    app: tauri::AppHandle,
    catalog: Vec<AgentProbe>,
) -> Result<AgentDiscoveryPayload, String> {
    let home = home_directory()?;
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from(""));
    let workspaces = project_workspaces(&app)?;
    let store_root = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT);
    let loaded = momonogi::store::load_manifest(&store_root);
    let (manifest, store_revision, store_etag, store_issue) = match &loaded {
        Ok(loaded) => (
            Some(&loaded.manifest),
            Some(loaded.manifest.revision),
            Some(loaded.etag.clone()),
            None,
        ),
        Err(error) => (None, None, None, Some(error.to_string())),
    };
    let agents = momonogi::discovery::discover_agents(DiscoveryInput {
        home: &home,
        path: &path,
        manifest,
        catalog: &catalog,
        openclaw_workspaces: &workspaces,
        codex_projects: &workspaces,
    });
    Ok(AgentDiscoveryPayload {
        agents,
        store_root: store_root.to_string_lossy().into_owned(),
        store_available: loaded.is_ok(),
        store_revision,
        store_etag,
        store_issue,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccessUpdatePayload {
    changed: bool,
    etag: String,
    revision: u64,
    writers: Vec<String>,
    readers: Vec<String>,
}

#[tauri::command]
fn set_agent_access(
    agent_id: String,
    role: Option<momonogi::store::AccessRole>,
    actor: String,
    if_match: String,
) -> Result<AccessUpdatePayload, String> {
    let root = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT)
        .canonicalize()
        .map_err(|error| format!("cannot open global store: {error}"))?;
    let mutation = momonogi::store::set_manifest_role(
        &root,
        &agent_id,
        role,
        &actor,
        &if_match,
    )
    .map_err(|error| error.to_string())?;
    Ok(AccessUpdatePayload {
        changed: mutation.changed,
        etag: mutation.loaded.etag,
        revision: mutation.loaded.manifest.revision,
        writers: mutation.loaded.manifest.writers,
        readers: mutation.loaded.manifest.readers,
    })
}

fn configuration_plan(
    app: &tauri::AppHandle,
    agent_id: &str,
) -> Result<Option<momonogi::configure::ConfigurationPlan>, String> {
    let Some(host) = momonogi::configure::Host::from_agent_id(agent_id) else {
        return Ok(None);
    };
    let home = home_directory()?;
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from(""));
    let tool = momo_executable(&home, &path);
    let workspaces = project_workspaces(app)?;
    let memory = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT);
    momonogi::configure::preview_host(&momonogi::configure::SyncOptions {
        host,
        home: &home,
        openclaw_workspaces: &workspaces,
        codex_projects: &workspaces,
        memory_root: &memory,
        hook_mode: "explicit",
        install_hooks: true,
        tool: &tool,
    })
    .map(Some)
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn preview_agent_configuration(
    app: tauri::AppHandle,
    agent_id: String,
) -> Result<Option<momonogi::configure::ConfigurationPlan>, String> {
    configuration_plan(&app, &agent_id)
}

#[tauri::command]
fn apply_agent_configuration(
    app: tauri::AppHandle,
    agent_id: String,
    digest: String,
) -> Result<momonogi::configure::ConfigurationApply, String> {
    let host = momonogi::configure::Host::from_agent_id(&agent_id)
        .ok_or_else(|| format!("agent {agent_id:?} has no Momonogi host adapter"))?;
    let home = home_directory()?;
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from(""));
    let tool = momo_executable(&home, &path);
    let workspaces = project_workspaces(&app)?;
    let memory = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT);
    momonogi::configure::apply_host(
        &momonogi::configure::SyncOptions {
            host,
            home: &home,
            openclaw_workspaces: &workspaces,
            codex_projects: &workspaces,
            memory_root: &memory,
            hook_mode: "explicit",
            install_hooks: true,
            tool: &tool,
        },
        &digest,
    )
    .map_err(|error| error.to_string())
}

fn registry_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("stores.json"))
        .map_err(|error| format!("cannot resolve application config directory: {error}"))
}

#[tauri::command]
fn get_store_registry(
    app: tauri::AppHandle,
) -> Result<Vec<momonogi::registry::StoreEntry>, String> {
    let registry = registry_path(&app)?;
    let global = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT);
    momonogi::registry::inspect_registry(&registry, &global).map_err(|error| error.to_string())
}

#[tauri::command]
fn register_project_store(
    app: tauri::AppHandle,
    project_path: String,
) -> Result<Vec<momonogi::registry::StoreEntry>, String> {
    let registry = registry_path(&app)?;
    let global = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT);
    momonogi::registry::register_project_store(&registry, &global, PathBuf::from(project_path).as_path())
        .map_err(|error| error.to_string())?;
    momonogi::registry::inspect_registry(&registry, &global).map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_project_store(
    app: tauri::AppHandle,
    project_path: String,
) -> Result<Vec<momonogi::registry::StoreEntry>, String> {
    let registry = registry_path(&app)?;
    let global = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT);
    momonogi::registry::remove_project_store(&registry, PathBuf::from(project_path).as_path())
        .map_err(|error| error.to_string())?;
    momonogi::registry::inspect_registry(&registry, &global).map_err(|error| error.to_string())
}

fn memory_sources(app: &tauri::AppHandle) -> Result<Vec<momonogi::explorer::StoreSource>, String> {
    let registry = registry_path(app)?;
    let global = momonogi::store::expand_path(momonogi::store::DEFAULT_GLOBAL_ROOT);
    momonogi::explorer::registry_sources(&registry, &global).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_memory_index(
    app: tauri::AppHandle,
    filter: momonogi::explorer::MemoryFilter,
) -> Result<momonogi::explorer::MemoryIndex, String> {
    Ok(momonogi::explorer::index_memories(
        &memory_sources(&app)?,
        &filter,
    ))
}

#[tauri::command]
fn get_memory_detail(
    app: tauri::AppHandle,
    store_path: String,
    slug: String,
    archived: bool,
) -> Result<momonogi::explorer::MemoryDetail, String> {
    let requested = PathBuf::from(store_path);
    let source = memory_sources(&app)?
        .into_iter()
        .find(|source| source.path == requested)
        .ok_or_else(|| "memory store is not registered".to_owned())?;
    momonogi::explorer::read_memory(&source, &slug, archived).map_err(|error| error.to_string())
}

#[tauri::command]
fn change_memory_tag(
    app: tauri::AppHandle,
    store_path: String,
    slug: String,
    tag: String,
    action: momonogi::tag::TagAction,
    actor: String,
    if_match: String,
) -> Result<momonogi::tag::TagMutation, String> {
    let requested = PathBuf::from(store_path);
    let source = memory_sources(&app)?
        .into_iter()
        .find(|source| source.path == requested)
        .ok_or_else(|| "memory store is not registered".to_owned())?;
    momonogi::tag::change_tag(
        &source.path,
        &slug,
        &tag,
        action,
        &actor,
        &if_match,
    )
    .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            discover_agents,
            set_agent_access,
            preview_agent_configuration,
            apply_agent_configuration,
            get_store_registry,
            register_project_store,
            remove_project_store,
            get_memory_index,
            get_memory_detail,
            change_memory_tag
        ])
        .run(tauri::generate_context!())
        .expect("error while running Momonogi Desktop");
}
