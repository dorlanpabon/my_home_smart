use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::{stream, StreamExt, TryStreamExt};
use reqwest::Method;
use serde_json::{json, Value};
use tokio::time::sleep;

use crate::{
    errors::{AppError, AppResult},
    models::{
        app::{
            ActionLogEntry, AppConfig, ConnectionTestResult, Device, LocalMetadata,
            ToggleChannelPayload, ToggleChannelResult,
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

        devices.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
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
        let command = json!({
            "commands": [
                {
                    "code": payload.channel_code,
                    "value": payload.value
                }
            ]
        });

        let command_paths = [
            format!("/v1.0/devices/{}/commands", payload.device_id),
            format!("/v1.0/iot-03/devices/{}/commands", payload.device_id),
        ];

        let mut command_sent = false;
        for path in command_paths {
            if self
                .authorized_request(Method::POST, &path, BTreeMap::new(), Some(command.clone()))
                .await
                .is_ok()
            {
                command_sent = true;
                break;
            }
        }

        if !command_sent {
            return Err(AppError::UnexpectedResponse(
                "None of the supported command endpoints accepted the request.".into(),
            ));
        }

        let statuses = self
            .fetch_confirmed_channel_status(
                &payload.device_id,
                &payload.channel_code,
                payload.value,
            )
            .await;

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

    async fn fetch_confirmed_channel_status(
        &self,
        device_id: &str,
        channel_code: &str,
        expected_value: bool,
    ) -> Vec<TuyaStatus> {
        let mut last_statuses = Vec::new();

        for delay_ms in [0_u64, 120, 250, 450] {
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }

            if let Ok(statuses) = self.fetch_device_status(device_id).await {
                let is_confirmed = statuses
                    .iter()
                    .find(|entry| entry.code == channel_code)
                    .and_then(|entry| status_value_as_bool(&entry.value))
                    == Some(expected_value);

                last_statuses = statuses;
                if is_confirmed {
                    return last_statuses;
                }
            }
        }

        ensure_channel_status(last_statuses, channel_code, expected_value)
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
    use serde_json::json;

    use crate::models::tuya::TuyaStatus;

    use super::{ensure_channel_status, status_value_as_bool};

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
}
