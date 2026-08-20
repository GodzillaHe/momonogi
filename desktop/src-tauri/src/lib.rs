use serde::Serialize;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![bootstrap])
        .run(tauri::generate_context!())
        .expect("error while running Momonogi Desktop");
}
