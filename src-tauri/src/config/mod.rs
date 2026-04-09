use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use tauri::{AppHandle, Manager};

use crate::{
    errors::AppResult,
    models::app::{
        ActionLogEntry, AppConfig, ChannelAlias, DeviceAlias, LocalMetadata,
        SaveUiPreferencesPayload, UiPreferences,
    },
};

const CONFIG_FILE: &str = "config.json";
const METADATA_FILE: &str = "metadata.json";
const ACTIONS_FILE: &str = "actions.jsonl";

pub struct LocalStore {
    app: AppHandle,
}

impl LocalStore {
    pub fn new(app: &AppHandle) -> Self {
        Self { app: app.clone() }
    }

    fn app_dir(&self) -> AppResult<PathBuf> {
        let dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|err| crate::errors::AppError::Io(err.to_string()))?;
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn file_path(&self, file_name: &str) -> AppResult<PathBuf> {
        Ok(self.app_dir()?.join(file_name))
    }

    pub fn load_config(&self) -> AppResult<Option<AppConfig>> {
        let path = self.file_path(CONFIG_FILE)?;
        if !path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(path)?;
        Ok(Some(serde_json::from_str::<AppConfig>(&raw)?))
    }

    pub fn save_config(&self, config: &AppConfig) -> AppResult<()> {
        let path = self.file_path(CONFIG_FILE)?;
        fs::write(path, serde_json::to_string_pretty(config)?)?;
        Ok(())
    }

    pub fn load_metadata(&self) -> AppResult<LocalMetadata> {
        let path = self.file_path(METADATA_FILE)?;
        if !path.exists() {
            return Ok(LocalMetadata::default());
        }

        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str::<LocalMetadata>(&raw)?)
    }

    pub fn save_metadata(&self, metadata: &LocalMetadata) -> AppResult<()> {
        let path = self.file_path(METADATA_FILE)?;
        fs::write(path, serde_json::to_string_pretty(metadata)?)?;
        Ok(())
    }

    pub fn save_device_alias(&self, alias: DeviceAlias) -> AppResult<LocalMetadata> {
        let mut metadata = self.load_metadata()?;
        metadata
            .device_aliases
            .retain(|entry| entry.device_id != alias.device_id);

        if !alias.alias.trim().is_empty() {
            metadata.device_aliases.push(alias);
        }

        self.save_metadata(&metadata)?;
        Ok(metadata)
    }

    pub fn save_channel_alias(&self, alias: ChannelAlias) -> AppResult<LocalMetadata> {
        let mut metadata = self.load_metadata()?;
        metadata.channel_aliases.retain(|entry| {
            !(entry.device_id == alias.device_id && entry.channel_code == alias.channel_code)
        });

        if !alias.alias.trim().is_empty() {
            metadata.channel_aliases.push(alias);
        }

        self.save_metadata(&metadata)?;
        Ok(metadata)
    }

    pub fn save_ui_preferences(
        &self,
        payload: &SaveUiPreferencesPayload,
    ) -> AppResult<UiPreferences> {
        let mut metadata = self.load_metadata()?;
        metadata.ui_preferences = UiPreferences {
            view_mode: payload.view_mode.trim().to_string(),
        };
        self.save_metadata(&metadata)?;
        Ok(metadata.ui_preferences)
    }

    pub fn load_action_log(&self) -> AppResult<Vec<ActionLogEntry>> {
        let path = self.file_path(ACTIONS_FILE)?;
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(entry) = serde_json::from_str::<ActionLogEntry>(&line) {
                entries.push(entry);
            }
        }

        entries.sort_by(|left, right| right.timestamp_ms.cmp(&left.timestamp_ms));
        entries.truncate(50);
        Ok(entries)
    }

    pub fn append_action_log(&self, entry: &ActionLogEntry) -> AppResult<()> {
        let path = self.file_path(ACTIONS_FILE)?;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", serde_json::to_string(entry)?)?;
        Ok(())
    }
}
