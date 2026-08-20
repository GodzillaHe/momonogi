use momonogi::discovery::{AgentProbe, DiscoveredAgent, DiscoveryInput};
use serde::Serialize;
use std::ffi::OsString;
use std::path::PathBuf;

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

#[tauri::command]
fn discover_agents(catalog: Vec<AgentProbe>) -> Result<AgentDiscoveryPayload, String> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| "cannot determine home directory".to_owned())?;
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from(""));
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
        openclaw_workspaces: &[],
        codex_projects: &[],
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            discover_agents,
            set_agent_access
        ])
        .run(tauri::generate_context!())
        .expect("error while running Momonogi Desktop");
}
