use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use futures::{stream, StreamExt, TryStreamExt};
use reqwest::Method;
use serde_json::{json, Value};

use crate::{
    errors::{AppError, AppResult},
    models::{
        app::{
            ActionLogEntry, AppConfig, ConnectionTestResult, Device, DeviceStatusUpdate,
            LocalMetadata, SetDeviceChannelsResult, ToggleChannelPayload, ToggleChannelResult,
        },
        tuya::{TuyaFunction, TuyaStatus},
    },
};

use super::{
    auth::{TokenCache, TuyaAuth},
    http_client::TuyaHttpClient,
    normalizer::normalize_device,
};

#[derive(Clone)]
pub struct TuyaService {
    http_client: TuyaHttpClient,
    auth: TuyaAuth,
}

impl TuyaService {
    pub fn new(config: AppConfig, token_cache: Arc<Mutex<Option<TokenCache>>>) -> Self {
        Self {
            http_client: TuyaHttpClient::new(config.clone()),
            auth: TuyaAuth::new(config, token_cache),
        }
    }

    pub async fn test_connection(&self) -> AppResult<ConnectionTestResult> {
        let _ = self.auth.get_token(false).await?;
        let devices = self.fetch_device_summaries().await?;
        Ok(ConnectionTestResult {
            success: true,
            message: "Connection successful.".into(),
            base_url: self.http_client.config().base_url.clone(),
            region_label: self.http_client.config().region_label.clone(),
            device_count: devices.len(),
        })
    }

    pub async fn list_devices(&self, metadata: &LocalMetadata) -> AppResult<Vec<Device>> {
        let summaries = self.fetch_device_summaries().await?;
        let metadata = metadata.clone();
        let concurrency = summaries.len().clamp(4, 12);

        let mut devices: Vec<Device> = stream::iter(summaries.into_iter().map(|summary| {
            let service = self.clone();
            let metadata = metadata.clone();
            async move { service.hydrate_device(summary, &metadata).await }
        }))
        .buffer_unordered(concurrency)
        .try_collect::<Vec<_>>()
        .await?;

        sort_devices_by_order(
            &mut devices,
            &metadata.ui_preferences.device_order,
            &metadata.ui_preferences.favorite_device_ids,
        );
        Ok(devices)
    }

    #[allow(dead_code)]
    pub async fn refresh_all_devices(&self, metadata: &LocalMetadata) -> AppResult<Vec<Device>> {
        self.list_devices(metadata).await
    }

    #[allow(dead_code)]
    pub async fn get_device_status(&self, device_id: &str) -> AppResult<Vec<TuyaStatus>> {
        self.fetch_device_status(device_id).await
    }

    #[allow(dead_code)]
    pub async fn get_device_functions(&self, device_id: &str) -> AppResult<Vec<TuyaFunction>> {
        self.fetch_device_functions(device_id).await
    }

    pub async fn toggle_channel(
        &self,
        metadata: &LocalMetadata,
        payload: ToggleChannelPayload,
    ) -> AppResult<ToggleChannelResult> {
        self.send_device_commands(
            &payload.device_id,
            vec![(payload.channel_code.clone(), payload.value)],
        )
        .await?;

        let statuses = ensure_channel_status(Vec::new(), &payload.channel_code, payload.value);

        let action_log_entry = ActionLogEntry {
            timestamp_ms: current_timestamp_ms(),
            action: if payload.value {
                "channel_on".into()
            } else {
                "channel_off".into()
            },
            device_id: Some(payload.device_id.clone()),
            device_name: metadata
                .device_alias_for(&payload.device_id)
                .map(str::to_string),
            channel_code: Some(payload.channel_code.clone()),
            success: true,
            message: format!(
                "{} {}",
                payload.channel_code,
                if payload.value {
                    "turned on"
                } else {
                    "turned off"
                }
            ),
        };

        Ok(ToggleChannelResult {
            device_id: payload.device_id,
            statuses,
            action_log_entry,
        })
    }

    pub async fn set_device_channels(
        &self,
        metadata: &LocalMetadata,
        device_id: &str,
        channel_codes: &[String],
        value: bool,
    ) -> AppResult<SetDeviceChannelsResult> {
        if channel_codes.is_empty() {
            return Err(AppError::UnexpectedResponse(
                "No controllable channels detected for this device.".into(),
            ));
        }

        self.send_device_commands(
            device_id,
            channel_codes
                .iter()
                .cloned()
                .map(|channel_code| (channel_code, value))
                .collect(),
        )
        .await?;

        let statuses = channel_codes
            .iter()
            .fold(Vec::new(), |current, channel_code| {
                ensure_channel_status(current, channel_code, value)
            });

        let action_log_entry = ActionLogEntry {
            timestamp_ms: current_timestamp_ms(),
            action: if value {
                "device_channels_on".into()
            } else {
                "device_channels_off".into()
            },
            device_id: Some(device_id.to_string()),
            device_name: metadata.device_alias_for(device_id).map(str::to_string),
            channel_code: None,
            success: true,
            message: format!(
                "{} channel(s) {}",
                channel_codes.len(),
                if value { "turned on" } else { "turned off" }
            ),
        };

        Ok(SetDeviceChannelsResult {
            device_id: device_id.to_string(),
            statuses,
            action_log_entry,
        })
    }

    pub async fn get_device_statuses(
        &self,
        device_ids: &[String],
    ) -> AppResult<Vec<DeviceStatusUpdate>> {
        let concurrency = device_ids.len().clamp(4, 12);

        let updates = stream::iter(device_ids.iter().cloned().map(|device_id| {
            let service = self.clone();
            async move {
                service
                    .fetch_device_status(&device_id)
                    .await
                    .map(|statuses| DeviceStatusUpdate {
                        device_id,
                        statuses,
                    })
            }
        }))
        .buffer_unordered(concurrency)
        .filter_map(|result| async move { result.ok() })
        .collect::<Vec<_>>()
        .await;

        Ok(updates)
    }

    async fn hydrate_device(&self, summary: Value, metadata: &LocalMetadata) -> AppResult<Device> {
        let id = first_string(&summary, &["id", "device_id"]).unwrap_or_else(|| "unknown".into());
        let status_future = self.fetch_device_status(&id);
        let functions_future = self.fetch_device_functions(&id);
        let (status_result, functions_result): (
            AppResult<Vec<TuyaStatus>>,
            AppResult<Vec<TuyaFunction>>,
        ) = futures::join!(status_future, functions_future);

        let status = status_result.unwrap_or_default();
        let functions = functions_result.unwrap_or_default();
        let capabilities = if functions.is_empty() {
            self.fetch_device_capabilities(&id)
                .await
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(normalize_device(
            summary,
            Value::Null,
            functions,
            status,
            capabilities,
            Value::Null,
            metadata,
        ))
    }

    async fn fetch_device_summaries(&self) -> AppResult<Vec<Value>> {
        let endpoints = [
            (
                "/v1.0/devices".to_string(),
                BTreeMap::<String, String>::new(),
            ),
            (
                "/v1.3/iot-03/devices".to_string(),
                map_query([("page_size", "200"), ("page_no", "1")]),
            ),
            (
                "/v1.0/iot-01/associated-users/devices".to_string(),
                map_query([("size", "200")]),
            ),
        ];

        let mut last_error = None;
        for (path, query) in endpoints {
            match self
                .authorized_request(Method::GET, &path, query, None)
                .await
            {
                Ok(value) => {
                    let parsed = extract_device_list(value);
                    if !parsed.is_empty() {
                        return Ok(parsed);
                    }
                }
                Err(err) => last_error = Some(err),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AppError::UnexpectedResponse(
                "No device listing endpoint returned a usable payload.".into(),
            )
        }))
    }

    async fn fetch_device_status(&self, device_id: &str) -> AppResult<Vec<TuyaStatus>> {
        let result = self
            .authorized_request(
                Method::GET,
                &format!("/v1.0/devices/{device_id}/status"),
                BTreeMap::new(),
                None,
            )
            .await?;

        extract_status_list(result)
    }

    async fn fetch_device_functions(&self, device_id: &str) -> AppResult<Vec<TuyaFunction>> {
        let endpoints = [
            format!("/v1.0/iot-03/devices/{device_id}/functions"),
            format!("/v1.0/devices/{device_id}/functions"),
        ];

        for path in endpoints {
            if let Ok(result) = self
                .authorized_request(Method::GET, &path, BTreeMap::new(), None)
                .await
            {
                let parsed = extract_functions_list(result);
                if !parsed.is_empty() {
                    return Ok(parsed);
                }
            }
        }

        Ok(Vec::new())
    }

    async fn fetch_device_capabilities(&self, device_id: &str) -> AppResult<Vec<TuyaFunction>> {
        let endpoints = [
            format!("/v1.0/iot-03/devices/{device_id}/capabilities-definition"),
            format!("/v1.0/devices/{device_id}/capabilities"),
        ];

        for path in endpoints {
            if let Ok(result) = self
                .authorized_request(Method::GET, &path, BTreeMap::new(), None)
                .await
            {
                let parsed = extract_functions_list(result);
                if !parsed.is_empty() {
                    return Ok(parsed);
                }
            }
        }

        Ok(Vec::new())
    }

    async fn authorized_request(
        &self,
        method: Method,
        path: &str,
        query: BTreeMap<String, String>,
        body: Option<Value>,
    ) -> AppResult<Value> {
        let token = self.auth.get_token(false).await?;
        match self
            .http_client
            .request_json(
                method.clone(),
                path,
                query.clone(),
                body.clone(),
                Some(&token),
            )
            .await
        {
            Ok(value) => Ok(value),
            Err(AppError::TokenExpired) => {
                self.auth.clear_token()?;
                let token = self.auth.get_token(true).await?;
                self.http_client
                    .request_json(method, path, query, body, Some(&token))
                    .await
            }
            Err(err) => Err(err),
        }
    }

    async fn send_device_commands(
        &self,
        device_id: &str,
        commands: Vec<(String, bool)>,
    ) -> AppResult<()> {
        let command = json!({
            "commands": commands
                .into_iter()
                .map(|(code, value)| json!({ "code": code, "value": value }))
                .collect::<Vec<_>>()
        });

        let command_paths = [
            format!("/v1.0/devices/{device_id}/commands"),
            format!("/v1.0/iot-03/devices/{device_id}/commands"),
        ];

        for path in command_paths {
            if self
                .authorized_request(Method::POST, &path, BTreeMap::new(), Some(command.clone()))
                .await
                .is_ok()
            {
                return Ok(());
            }
        }

        Err(AppError::UnexpectedResponse(
            "None of the supported command endpoints accepted the request.".into(),
        ))
    }
}

fn extract_device_list(value: Value) -> Vec<Value> {
    if let Some(array) = value.as_array() {
        return array.clone();
    }

    if let Some(list) = value.get("list").and_then(Value::as_array) {
        return list.clone();
    }

    if let Some(devices) = value.get("devices").and_then(Value::as_array) {
        return devices.clone();
    }

    if let Some(result) = value.get("result").and_then(Value::as_array) {
        return result.clone();
    }

    Vec::new()
}

fn extract_status_list(value: Value) -> AppResult<Vec<TuyaStatus>> {
    let array = value
        .as_array()
        .cloned()
        .or_else(|| value.get("status").and_then(Value::as_array).cloned())
        .ok_or_else(|| AppError::UnexpectedResponse(value.to_string()))?;

    array
        .into_iter()
        .map(|entry| {
            Ok(TuyaStatus {
                code: first_string(&entry, &["code"]).unwrap_or_default(),
                value: entry.get("value").cloned().unwrap_or(Value::Null),
            })
        })
        .collect()
}

fn extract_functions_list(value: Value) -> Vec<TuyaFunction> {
    let array = value
        .as_array()
        .cloned()
        .or_else(|| value.get("functions").and_then(Value::as_array).cloned())
        .or_else(|| value.get("status").and_then(Value::as_array).cloned())
        .or_else(|| value.get("capabilities").and_then(Value::as_array).cloned())
        .unwrap_or_default();

    array
        .into_iter()
        .map(|entry| TuyaFunction {
            code: first_string(&entry, &["code"]).unwrap_or_default(),
            value_type: first_string(&entry, &["type", "valueType"]),
            values: entry.get("values").cloned(),
            mode: first_string(&entry, &["mode"]),
            support: first_string(&entry, &["support"]),
            name: first_string(&entry, &["name", "display_name"]),
            description: first_string(&entry, &["desc", "description"]),
        })
        .filter(|entry| !entry.code.is_empty())
        .collect()
}

fn ensure_channel_status(
    mut statuses: Vec<TuyaStatus>,
    channel_code: &str,
    expected_value: bool,
) -> Vec<TuyaStatus> {
    if let Some(status) = statuses.iter_mut().find(|entry| entry.code == channel_code) {
        status.value = Value::Bool(expected_value);
        return statuses;
    }

    statuses.push(TuyaStatus {
        code: channel_code.to_string(),
        value: Value::Bool(expected_value),
    });
    statuses
}

#[cfg(test)]
fn status_value_as_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(inner) => Some(*inner),
        Value::String(inner) if inner.eq_ignore_ascii_case("true") => Some(true),
        Value::String(inner) if inner.eq_ignore_ascii_case("false") => Some(false),
        _ => None,
    }
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn map_query<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<String, String> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::models::app::Device;
    use serde_json::json;

    use crate::models::tuya::TuyaStatus;

    use super::{ensure_channel_status, sort_devices_by_order, status_value_as_bool};

    #[test]
    fn ensure_channel_status_overwrites_stale_value() {
        let statuses = vec![
            TuyaStatus {
                code: "switch_1".into(),
                value: json!(true),
            },
            TuyaStatus {
                code: "switch_2".into(),
                value: json!(false),
            },
        ];

        let result = ensure_channel_status(statuses, "switch_1", false);
        let switch_1 = result
            .iter()
            .find(|entry| entry.code == "switch_1")
            .and_then(|entry| status_value_as_bool(&entry.value));

        assert_eq!(switch_1, Some(false));
    }

    #[test]
    fn status_value_as_bool_supports_string_bools() {
        assert_eq!(status_value_as_bool(&json!("true")), Some(true));
        assert_eq!(status_value_as_bool(&json!("false")), Some(false));
        assert_eq!(status_value_as_bool(&json!("other")), None);
    }

    #[test]
    fn sort_devices_by_order_prioritizes_saved_device_order() {
        let mut devices = vec![make_device("b", "Beta"), make_device("a", "Alpha")];

        sort_devices_by_order(&mut devices, &["b".into()], &[]);

        assert_eq!(devices[0].id, "b");
        assert_eq!(devices[1].id, "a");
    }

    #[test]
    fn sort_devices_by_order_prioritizes_favorites_before_saved_order() {
        let mut devices = vec![
            make_device("c", "Charlie"),
            make_device("b", "Beta"),
            make_device("a", "Alpha"),
        ];

        sort_devices_by_order(&mut devices, &["b".into()], &["c".into()]);

        assert_eq!(devices[0].id, "c");
        assert_eq!(devices[1].id, "b");
        assert_eq!(devices[2].id, "a");
    }

    fn make_device(id: &str, name: &str) -> Device {
        Device {
            id: id.into(),
            name: name.into(),
            online: true,
            category: None,
            product_id: None,
            inferred_type: "switch".into(),
            gang_count: 1,
            channels: Vec::new(),
            raw: crate::models::app::RawDeviceData {
                summary: json!({ "name": name }),
                details: json!({}),
                functions: Vec::new(),
                status: Vec::new(),
                capabilities: Vec::new(),
                specifications: json!({}),
            },
            metadata: None,
        }
    }
}

fn sort_devices_by_order(
    devices: &mut [Device],
    device_order: &[String],
    favorite_device_ids: &[String],
) {
    let order_index = device_order
        .iter()
        .enumerate()
        .map(|(index, device_id)| (device_id.as_str(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let favorite_set = favorite_device_ids
        .iter()
        .map(|device_id| device_id.as_str())
        .collect::<std::collections::HashSet<_>>();

    devices.sort_by(|left, right| {
        match (
            favorite_set.contains(left.id.as_str()),
            favorite_set.contains(right.id.as_str()),
        ) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        match (
            order_index.get(left.id.as_str()),
            order_index.get(right.id.as_str()),
        ) {
            (Some(left_index), Some(right_index)) => left_index.cmp(right_index),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        }
    });
}
