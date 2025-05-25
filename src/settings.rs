use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Language;
use crate::config::RefreshInterval;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub language: String,
    pub refresh_interval: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { language: Language::Chinese.to_string(), refresh_interval: RefreshInterval::Min5.as_secs() }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)?;
        let settings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| anyhow!("Cannot get configuration directory"))?;

    Ok(config_dir.join("CursorBarWatch").join("settings.json"))
}
