#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTaskPlan {
    pub id: String,
    pub cron_like_expression: String,
    pub action: String,
}

pub struct SchedulerModuleStub;

impl SchedulerModuleStub {
    pub fn summary(&self) -> &'static str {
        "Scheduler placeholder. Future execution should orchestrate domain actions without coupling to the desktop UI."
    }
}
