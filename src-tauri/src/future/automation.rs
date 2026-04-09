#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger: String,
    pub action: String,
}

pub struct AutomationModuleStub;

impl AutomationModuleStub {
    pub fn summary(&self) -> &'static str {
        "Automation module placeholder. Future rules should call the same domain services used by Tauri commands."
    }
}
