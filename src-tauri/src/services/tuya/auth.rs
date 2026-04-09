use std::sync::{Arc, Mutex};

use reqwest::Method;
use serde_json::Value;

use crate::{
    errors::{AppError, AppResult},
    models::app::AppConfig,
};

use super::http_client::TuyaHttpClient;

#[derive(Debug, Clone)]
pub struct TokenCache {
    pub access_token: String,
    pub expires_at_ms: u64,
}

#[derive(Clone)]
pub struct TuyaAuth {
    http_client: TuyaHttpClient,
    token_cache: Arc<Mutex<Option<TokenCache>>>,
}

impl TuyaAuth {
    pub fn new(config: AppConfig, token_cache: Arc<Mutex<Option<TokenCache>>>) -> Self {
        Self {
            http_client: TuyaHttpClient::new(config),
            token_cache,
        }
    }

    pub async fn get_token(&self, force_refresh: bool) -> AppResult<String> {
        if !force_refresh {
            if let Some(token) = self.cached_token()? {
                return Ok(token);
            }
        }

        let result = self
            .http_client
            .request_json(
                Method::GET,
                "/v1.0/token",
                query_pairs([("grant_type", "1")]),
                None,
                None,
            )
            .await?;

        let access_token = result
            .get("access_token")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::UnexpectedResponse(result.to_string()))?
            .to_string();

        let expire_time_seconds = result
            .get("expire_time")
            .and_then(Value::as_u64)
            .unwrap_or(3600);
        let now = current_timestamp_ms();
        let expires_at_ms = now + expire_time_seconds.saturating_sub(60) * 1_000;
        let cached = TokenCache {
            access_token: access_token.clone(),
            expires_at_ms,
        };

        *self.token_cache.lock().map_err(|_| AppError::Lock)? = Some(cached);
        Ok(access_token)
    }

    pub fn clear_token(&self) -> AppResult<()> {
        *self.token_cache.lock().map_err(|_| AppError::Lock)? = None;
        Ok(())
    }

    fn cached_token(&self) -> AppResult<Option<String>> {
        let cached = self.token_cache.lock().map_err(|_| AppError::Lock)?;
        if let Some(cache) = cached.as_ref() {
            if cache.expires_at_ms > current_timestamp_ms() {
                return Ok(Some(cache.access_token.clone()));
            }
        }

        Ok(None)
    }
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn query_pairs<const N: usize>(
    pairs: [(&str, &str); N],
) -> std::collections::BTreeMap<String, String> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::TokenCache;

    #[test]
    fn token_cache_is_cloneable() {
        let token = TokenCache {
            access_token: "abc".into(),
            expires_at_ms: 123,
        };

        assert_eq!(token.clone().access_token, "abc");
    }
}
