use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub const TICK_RATE_MS: u64 = 33; // ~30 FPS
pub const AUTOSAVE_INTERVAL_SECS: u64 = 15;
pub const GRAVITY: f64 = 38.0; // cells per second^2
pub const RESTITUTION: f64 = 0.65; // bounciness on floor/wall impact
pub const FRICTION: f64 = 0.92;
pub const IDLE_THRESHOLD_SECS: i64 = 180; // 3 minutes idle triggers boredom/sleepiness

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub data_file: PathBuf,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "moodshell", "MoodShell")
            .context("Could not determine home directory for MoodShell data")?;
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).context("Failed to create MoodShell data directory")?;

        Ok(Self {
            data_file: data_dir.join("state.json"),
        })
    }
}
