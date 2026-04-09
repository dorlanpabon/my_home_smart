#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalEndpointPlan {
    pub method: String,
    pub path: String,
    pub description: String,
}

pub struct LocalApiModuleStub;

impl LocalApiModuleStub {
    pub fn default_routes(&self) -> Vec<LocalEndpointPlan> {
        vec![
            LocalEndpointPlan {
                method: "POST".into(),
                path: "/devices/:id/channels/:channelCode/on".into(),
                description: "Turn on one Tuya channel using the existing device action service."
                    .into(),
            },
            LocalEndpointPlan {
                method: "POST".into(),
                path: "/devices/:id/channels/:channelCode/off".into(),
                description: "Turn off one Tuya channel using the existing device action service."
                    .into(),
            },
        ]
    }
}
