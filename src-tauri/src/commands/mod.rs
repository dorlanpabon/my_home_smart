use tauri::{AppHandle, State};

use crate::{
    config::LocalStore,
    errors::{AppError, AppErrorPayload},
    models::app::{
        ActionLogEntry, AppConfig, BootstrapPayload, ChannelAlias, ConnectionStatus,
        SaveChannelAliasPayload, SaveDeviceAliasPayload, SaveUiPreferencesPayload,
        ToggleChannelPayload, ToggleChannelResult,
    },
    services::tuya::service::TuyaService,
    SharedState,
};

#[tauri::command]
pub async fn load_bootstrap(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<BootstrapPayload, AppErrorPayload> {
    let store = LocalStore::new(&app);
    let mut metadata = store.load_metadata().map_err(AppErrorPayload::from)?;
    metadata.ui_preferences.view_mode = normalize_view_mode(&metadata.ui_preferences.view_mode);
    let action_log = store.load_action_log().map_err(AppErrorPayload::from)?;

    let Some(config) = store.load_config().map_err(AppErrorPayload::from)? else {
        return Ok(BootstrapPayload {
            has_config: false,
            config: None,
            ui_preferences: metadata.ui_preferences,
            action_log,
            devices: Vec::new(),
            connection: ConnectionStatus::needs_config(),
            uses_cached_devices: false,
        });
    };

    if let Some(snapshot) = store.load_cached_devices().map_err(AppErrorPayload::from)? {
        return Ok(BootstrapPayload {
            has_config: true,
            config: Some(config.masked()),
            ui_preferences: metadata.ui_preferences,
            action_log,
            devices: snapshot.devices,
            connection: ConnectionStatus {
                state: "connected".into(),
                message: Some("Loaded cached devices. Refreshing from Tuya Cloud.".into()),
                last_checked_at: Some(snapshot.updated_at_ms),
            },
            uses_cached_devices: true,
        });
    }

    let service = TuyaService::new(config.clone(), state.token_cache.clone());
    match service.list_devices(&metadata).await {
        Ok(devices) => {
            let _ = store.save_cached_devices(&devices);

            Ok(BootstrapPayload {
                has_config: true,
                config: Some(config.masked()),
                ui_preferences: metadata.ui_preferences,
                action_log,
                devices,
                connection: ConnectionStatus {
                    state: "connected".into(),
                    message: Some("Connected to Tuya Cloud.".into()),
                    last_checked_at: Some(current_timestamp_ms()),
                },
                uses_cached_devices: false,
            })
        }
        Err(err) => {
            let payload = AppErrorPayload::from(err);
            Ok(BootstrapPayload {
                has_config: true,
                config: Some(config.masked()),
                ui_preferences: metadata.ui_preferences,
                action_log,
                devices: Vec::new(),
                connection: ConnectionStatus {
                    state: "error".into(),
                    message: Some(payload.message),
                    last_checked_at: Some(current_timestamp_ms()),
                },
                uses_cached_devices: false,
            })
        }
    }
}

#[tauri::command]
pub async fn get_config_masked(
    app: AppHandle,
) -> Result<Option<crate::models::app::MaskedAppConfig>, AppErrorPayload> {
    let store = LocalStore::new(&app);
    let config = store.load_config().map_err(AppErrorPayload::from)?;
    Ok(config.map(|config| config.masked()))
}

#[tauri::command]
pub async fn save_config(app: AppHandle, payload: AppConfig) -> Result<(), AppErrorPayload> {
    let store = LocalStore::new(&app);
    let existing = store.load_config().map_err(AppErrorPayload::from)?;
    let merged = AppConfig {
        client_id: payload.client_id.trim().to_string(),
        client_secret: if payload.client_secret.trim().is_empty() {
            existing
                .as_ref()
                .map(|config| config.client_secret.clone())
                .unwrap_or_default()
        } else {
            payload.client_secret.trim().to_string()
        },
        base_url: payload.base_url.trim().to_string(),
        region_label: payload.region_label.trim().to_string(),
    };

    if !merged.is_complete() {
        return Err(AppErrorPayload::from(AppError::InvalidConfig));
    }

    store.save_config(&merged).map_err(AppErrorPayload::from)
}

#[tauri::command]
pub async fn test_connection(
    payload: AppConfig,
    state: State<'_, SharedState>,
) -> Result<crate::models::app::ConnectionTestResult, AppErrorPayload> {
    if !payload.is_complete() {
        return Err(AppErrorPayload::from(AppError::InvalidConfig));
    }

    let service = TuyaService::new(payload, state.token_cache.clone());
    service
        .test_connection()
        .await
        .map_err(AppErrorPayload::from)
}

#[tauri::command]
pub async fn list_devices(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<Vec<crate::models::app::Device>, AppErrorPayload> {
    load_devices(&app, &state)
        .await
        .map_err(AppErrorPayload::from)
}

#[tauri::command]
pub async fn refresh_all_devices(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<Vec<crate::models::app::Device>, AppErrorPayload> {
    load_devices(&app, &state)
        .await
        .map_err(AppErrorPayload::from)
}

#[tauri::command]
pub async fn toggle_channel(
    app: AppHandle,
    state: State<'_, SharedState>,
    payload: ToggleChannelPayload,
) -> Result<ToggleChannelResult, AppErrorPayload> {
    let store = LocalStore::new(&app);
    let config = store
        .load_config()
        .map_err(AppErrorPayload::from)?
        .ok_or_else(|| AppErrorPayload::from(AppError::MissingConfig))?;
    let metadata = store.load_metadata().map_err(AppErrorPayload::from)?;
    let service = TuyaService::new(config, state.token_cache.clone());
    let action = service
        .toggle_channel(&metadata, payload.clone())
        .await
        .map_err(AppErrorPayload::from)?;
    let _ = store.append_action_log(&action.action_log_entry);
    let _ = patch_cached_device_statuses(&store, &action.device_id, &action.statuses);

    Ok(action)
}

#[tauri::command]
pub async fn save_device_alias(
    app: AppHandle,
    payload: SaveDeviceAliasPayload,
) -> Result<(), AppErrorPayload> {
    let store = LocalStore::new(&app);
    let metadata = store
        .save_device_alias(crate::models::app::DeviceAlias {
            device_id: payload.device_id,
            alias: payload.alias,
        })
        .map_err(AppErrorPayload::from)?;
    let _ = patch_cached_device_alias(&store, &metadata);
    Ok(())
}

#[tauri::command]
pub async fn save_channel_alias(
    app: AppHandle,
    payload: SaveChannelAliasPayload,
) -> Result<(), AppErrorPayload> {
    let store = LocalStore::new(&app);
    let metadata = store
        .save_channel_alias(ChannelAlias {
            device_id: payload.device_id,
            channel_code: payload.channel_code,
            alias: payload.alias,
        })
        .map_err(AppErrorPayload::from)?;
    let _ = patch_cached_device_alias(&store, &metadata);
    Ok(())
}

#[tauri::command]
pub async fn save_ui_preferences(
    app: AppHandle,
    payload: SaveUiPreferencesPayload,
) -> Result<crate::models::app::UiPreferences, AppErrorPayload> {
    let store = LocalStore::new(&app);
    let payload = SaveUiPreferencesPayload {
        view_mode: normalize_view_mode(&payload.view_mode),
    };
    store
        .save_ui_preferences(&payload)
        .map_err(AppErrorPayload::from)
}

#[tauri::command]
pub async fn get_action_log(app: AppHandle) -> Result<Vec<ActionLogEntry>, AppErrorPayload> {
    let store = LocalStore::new(&app);
    store.load_action_log().map_err(AppErrorPayload::from)
}

async fn load_devices(
    app: &AppHandle,
    state: &State<'_, SharedState>,
) -> Result<Vec<crate::models::app::Device>, AppError> {
    let store = LocalStore::new(app);
    let config = store.load_config()?.ok_or(AppError::MissingConfig)?;
    let metadata = store.load_metadata()?;
    let service = TuyaService::new(config, state.token_cache.clone());
    let devices = service.list_devices(&metadata).await?;
    let _ = store.save_cached_devices(&devices);
    Ok(devices)
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn normalize_view_mode(value: &str) -> String {
    match value {
        "developer" | "detailed" => "developer".into(),
        "user" | "compact" => "user".into(),
        _ => "user".into(),
    }
}

fn patch_cached_device_statuses(
    store: &LocalStore,
    device_id: &str,
    statuses: &[crate::models::tuya::TuyaStatus],
) -> Result<(), AppError> {
    let Some(mut snapshot) = store.load_cached_devices()? else {
        return Ok(());
    };

    if let Some(device) = snapshot
        .devices
        .iter_mut()
        .find(|entry| entry.id == device_id)
    {
        for channel in &mut device.channels {
            if let Some(status) = statuses.iter().find(|entry| entry.code == channel.code) {
                if let Some(value) = parse_status_bool(&status.value) {
                    channel.current_state = Some(value);
                }
            }
        }

        device.raw.status = statuses.to_vec();
        store.save_cached_devices(&snapshot.devices)?;
    }

    Ok(())
}

fn patch_cached_device_alias(
    store: &LocalStore,
    metadata: &crate::models::app::LocalMetadata,
) -> Result<(), AppError> {
    let Some(mut snapshot) = store.load_cached_devices()? else {
        return Ok(());
    };

    for device in &mut snapshot.devices {
        device.metadata = Some(crate::models::app::DeviceLocalMetadata {
            alias: metadata.device_alias_for(&device.id).map(str::to_string),
        });

        device.name = metadata
            .device_alias_for(&device.id)
            .filter(|alias| !alias.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| resolve_cached_device_name(device));

        for channel in &mut device.channels {
            let alias = metadata
                .channel_alias_for(&device.id, &channel.code)
                .filter(|alias| !alias.is_empty())
                .map(str::to_string);
            channel.alias = alias.clone();
            channel.display_name =
                alias.unwrap_or_else(|| default_cached_channel_name(&channel.code, channel.index));
        }
    }

    store.save_cached_devices(&snapshot.devices)?;
    Ok(())
}

fn parse_status_bool(value: &serde_json::Value) -> Option<bool> {
    match value {
        serde_json::Value::Bool(inner) => Some(*inner),
        serde_json::Value::String(inner) if inner.eq_ignore_ascii_case("true") => Some(true),
        serde_json::Value::String(inner) if inner.eq_ignore_ascii_case("false") => Some(false),
        _ => None,
    }
}

fn resolve_cached_device_name(device: &crate::models::app::Device) -> String {
    device
        .raw
        .summary
        .get("name")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            device
                .raw
                .details
                .get("name")
                .and_then(serde_json::Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| device.name.clone())
}

fn default_cached_channel_name(code: &str, index: usize) -> String {
    match code {
        "switch" => "Main channel".into(),
        "switch_led" => "Backlight".into(),
        _ if code.starts_with("switch_") => format!("Switch {index}"),
        _ => code.to_string(),
    }
}
