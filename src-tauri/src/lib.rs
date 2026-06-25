mod commands;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;

pub type JobStore = Arc<Mutex<HashMap<String, CommandChild>>>;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .manage(Arc::new(Mutex::new(HashMap::<String, CommandChild>::new())) as JobStore)
        .invoke_handler(tauri::generate_handler![
            commands::probe_video,
            commands::start_process,
            commands::cancel_job,
            commands::reveal_in_folder,
            commands::list_vaapi_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
