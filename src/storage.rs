use crate::personality::PersonalityMemory;
use crate::pet::Pet;
use crate::tasks::TaskManager;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    pub pet: Pet,
    pub tasks: TaskManager,
    pub personality: PersonalityMemory,
}

impl AppState {
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            let default_state = Self::default();
            default_state.save(path)?;
            return Ok(default_state);
        }

        let raw = fs::read_to_string(path).context("Failed to read state file")?;
        let state: AppState =
            serde_json::from_str(&raw).context("Failed to parse MoodShell JSON state")?;
        Ok(state)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, json).context("Failed to write temp state file")?;
        fs::rename(&tmp_path, path).context("Failed to atomically replace state file")?;
        Ok(())
    }
}
