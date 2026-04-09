use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration is missing. Save your Tuya credentials first.")]
    MissingConfig,
    #[error("The saved configuration is incomplete.")]
    InvalidConfig,
    #[error("Local storage error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Tuya API error {code}: {message}")]
    TuyaApi { code: String, message: String },
    #[error("Tuya access token expired or is invalid.")]
    TokenExpired,
    #[error("Unexpected Tuya response: {0}")]
    UnexpectedResponse(String),
    #[error("Internal synchronization error.")]
    Lock,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorPayload {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl From<AppError> for AppErrorPayload {
    fn from(value: AppError) -> Self {
        match value {
            AppError::MissingConfig => Self {
                code: "missing_config".into(),
                message: value.to_string(),
                details: None,
            },
            AppError::InvalidConfig => Self {
                code: "invalid_config".into(),
                message: value.to_string(),
                details: None,
            },
            AppError::Io(details) => Self {
                code: "io".into(),
                message: "Unable to access local app data.".into(),
                details: Some(details),
            },
            AppError::Serialization(details) => Self {
                code: "serialization".into(),
                message: "Unable to serialize or parse application data.".into(),
                details: Some(details),
            },
            AppError::Http(details) => Self {
                code: "http".into(),
                message: "Unable to reach the Tuya Cloud API.".into(),
                details: Some(details),
            },
            AppError::TuyaApi { code, message } => Self {
                code,
                message,
                details: None,
            },
            AppError::TokenExpired => Self {
                code: "token_expired".into(),
                message: "The Tuya token expired and could not be refreshed.".into(),
                details: None,
            },
            AppError::UnexpectedResponse(details) => Self {
                code: "unexpected_response".into(),
                message: "The Tuya API returned an unexpected response.".into(),
                details: Some(details),
            },
            AppError::Lock => Self {
                code: "lock".into(),
                message: "Internal application state is not available.".into(),
                details: None,
            },
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value.to_string())
    }
}
