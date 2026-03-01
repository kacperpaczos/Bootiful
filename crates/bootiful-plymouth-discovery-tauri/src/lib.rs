use tauri::Runtime;
use bootiful_plymouth_discovery::{collect_all, model::PlymouthConfig};

#[tauri::command]
pub fn get_plymouth_config<R: Runtime>() -> Result<PlymouthConfig, String> {
    collect_all().map_err(|e| e.to_string())
}

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("plymouth-discovery")
        .invoke_handler(tauri::generate_handler![get_plymouth_config])
        .build()
}
