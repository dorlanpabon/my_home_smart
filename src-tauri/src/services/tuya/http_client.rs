use std::collections::BTreeMap;

use reqwest::{Client, Method};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::app::AppConfig,
};

use super::signing::{sign, string_to_sign};

#[derive(Clone)]
pub struct TuyaHttpClient {
    client: Client,
    config: AppConfig,
}

impl TuyaHttpClient {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    fn normalize_base_url(&self) -> String {
        self.config.base_url.trim_end_matches('/').to_string()
    }

    fn canonical_url(path: &str, query: &BTreeMap<String, String>) -> String {
        if query.is_empty() {
            return path.to_string();
        }

        let query_string = query
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&");
        format!("{path}?{query_string}")
    }

    pub async fn request_json(
        &self,
        method: Method,
        path: &str,
        query: BTreeMap<String, String>,
        body: Option<Value>,
        access_token: Option<&str>,
    ) -> AppResult<Value> {
        let canonical_url = Self::canonical_url(path, &query);
        let url = format!("{}{}", self.normalize_base_url(), canonical_url);
        let body_string = body.as_ref().map(Value::to_string).unwrap_or_default();
        let timestamp = current_timestamp_ms().to_string();
        let nonce = Uuid::new_v4().to_string();
        let signature = sign(
            &self.config.client_id,
            &self.config.client_secret,
            access_token,
            &timestamp,
            &nonce,
            &string_to_sign(method.as_str(), &body_string, &canonical_url),
        )?;

        let mut request = self
            .client
            .request(method, url)
            .header("client_id", &self.config.client_id)
            .header("sign", signature)
            .header("t", &timestamp)
            .header("nonce", &nonce)
            .header("sign_method", "HMAC-SHA256")
            .header("Content-Type", "application/json");

        if let Some(token) = access_token {
            request = request.header("access_token", token);
        }

        if !query.is_empty() {
            request = request.query(&query);
        }

        if let Some(body) = body {
            request = request.body(body.to_string());
        }

        let response = request.send().await?;
        let response_text = response.text().await?;
        let payload = serde_json::from_str::<Value>(&response_text)?;

        if payload
            .get("success")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Ok(payload.get("result").cloned().unwrap_or(Value::Null));
        }

        let code = json_value_to_string(payload.get("code"));
        let message = payload
            .get("msg")
            .and_then(Value::as_str)
            .or_else(|| payload.get("message").and_then(Value::as_str))
            .unwrap_or("Unknown Tuya API error")
            .to_string();

        if is_token_expired(&code, &message) {
            return Err(AppError::TokenExpired);
        }

        Err(AppError::TuyaApi { code, message })
    }
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn json_value_to_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(inner)) => inner.clone(),
        Some(Value::Number(inner)) => inner.to_string(),
        Some(Value::Bool(inner)) => inner.to_string(),
        _ => "unknown".into(),
    }
}

fn is_token_expired(code: &str, message: &str) -> bool {
    matches!(code, "1010" | "1011" | "1012" | "1013" | "1014")
        || message.to_ascii_lowercase().contains("token")
}
