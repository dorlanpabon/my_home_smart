mod commands;
mod config;
mod errors;
mod future;
mod models;
mod services;

use std::sync::{Arc, Mutex};

use commands::{
    get_action_log, get_config_masked, list_devices, load_bootstrap, refresh_all_devices,
    refresh_device_statuses, save_channel_alias, save_config, save_device_alias,
    save_ui_preferences, set_device_channels, test_connection, toggle_channel,
};
use services::tuya::auth::TokenCache;

#[derive(Clone)]
pub struct SharedState {
    pub token_cache: Arc<Mutex<Option<TokenCache>>>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            token_cache: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SharedState::default())
        .invoke_handler(tauri::generate_handler![
            load_bootstrap,
            get_config_masked,
            save_config,
            test_connection,
            list_devices,
            refresh_all_devices,
            refresh_device_statuses,
            toggle_channel,
            set_device_channels,
            save_device_alias,
            save_channel_alias,
            save_ui_preferences,
            get_action_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
