use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::tuya::{TuyaFunction, TuyaStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub base_url: String,
    pub region_label: String,
}

impl AppConfig {
    pub fn is_complete(&self) -> bool {
        !self.client_id.trim().is_empty()
            && !self.client_secret.trim().is_empty()
            && !self.base_url.trim().is_empty()
            && !self.region_label.trim().is_empty()
    }

    pub fn masked(&self) -> MaskedAppConfig {
        let suffix = self
            .client_secret
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();

        MaskedAppConfig {
            client_id: self.client_id.clone(),
            client_secret_masked: if self.client_secret.is_empty() {
                String::new()
            } else {
                format!("****{}", suffix)
            },
            client_secret_present: !self.client_secret.is_empty(),
            base_url: self.base_url.clone(),
            region_label: self.region_label.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskedAppConfig {
    pub client_id: String,
    pub client_secret_masked: String,
    pub client_secret_present: bool,
    pub base_url: String,
    pub region_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAlias {
    pub device_id: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelAlias {
    pub device_id: String,
    pub channel_code: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiPreferences {
    pub view_mode: String,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            view_mode: "user".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMetadata {
    pub device_aliases: Vec<DeviceAlias>,
    pub channel_aliases: Vec<ChannelAlias>,
    #[serde(default)]
    pub ui_preferences: UiPreferences,
}

impl LocalMetadata {
    pub fn device_alias_for(&self, device_id: &str) -> Option<&str> {
        self.device_aliases
            .iter()
            .find(|entry| entry.device_id == device_id && !entry.alias.trim().is_empty())
            .map(|entry| entry.alias.as_str())
    }

    pub fn channel_alias_for(&self, device_id: &str, channel_code: &str) -> Option<&str> {
        self.channel_aliases
            .iter()
            .find(|entry| {
                entry.device_id == device_id
                    && entry.channel_code == channel_code
                    && !entry.alias.trim().is_empty()
            })
            .map(|entry| entry.alias.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLocalMetadata {
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceChannel {
    pub code: String,
    pub display_name: String,
    pub index: usize,
    pub current_state: Option<bool>,
    pub controllable: bool,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawDeviceData {
    pub summary: Value,
    pub details: Value,
    pub functions: Vec<TuyaFunction>,
    pub status: Vec<TuyaStatus>,
    pub capabilities: Vec<TuyaFunction>,
    pub specifications: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub name: String,
    pub online: bool,
    pub category: Option<String>,
    pub product_id: Option<String>,
    pub inferred_type: String,
    pub gang_count: usize,
    pub channels: Vec<DeviceChannel>,
    pub raw: RawDeviceData,
    pub metadata: Option<DeviceLocalMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub state: String,
    pub message: Option<String>,
    pub last_checked_at: Option<u64>,
}

impl ConnectionStatus {
    pub fn needs_config() -> Self {
        Self {
            state: "needs_config".into(),
            message: Some("Save your Tuya credentials to continue.".into()),
            last_checked_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub has_config: bool,
    pub config: Option<MaskedAppConfig>,
    pub ui_preferences: UiPreferences,
    pub action_log: Vec<ActionLogEntry>,
    pub devices: Vec<Device>,
    pub connection: ConnectionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub base_url: String,
    pub region_label: String,
    pub device_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionLogEntry {
    pub timestamp_ms: u64,
    pub action: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub channel_code: Option<String>,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleChannelPayload {
    pub device_id: String,
    pub channel_code: String,
    pub value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleChannelResult {
    pub device_id: String,
    pub statuses: Vec<TuyaStatus>,
    pub action_log_entry: ActionLogEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDeviceAliasPayload {
    pub device_id: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveChannelAliasPayload {
    pub device_id: String,
    pub channel_code: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveUiPreferencesPayload {
    pub view_mode: String,
}
