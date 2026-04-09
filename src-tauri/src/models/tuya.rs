use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuyaFunction {
    pub code: String,
    pub value_type: Option<String>,
    pub values: Option<Value>,
    pub mode: Option<String>,
    pub support: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuyaStatus {
    pub code: String,
    pub value: Value,
}
