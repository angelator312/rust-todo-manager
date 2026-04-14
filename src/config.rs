use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Maps project alias → path to todo file
pub type ProjectShortcuts = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub projects: ProjectShortcuts,
}

impl Config {
    /// Load config from "~/.config/todo-manager/config.json"
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing config or return empty default
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Save empty config for user to populate
            let default = Self::default();
            default.save()?;
            eprintln!("✨ Created config at: {}", config_path.display());
            Ok(default)
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        fs::write(&config_path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Add or update a project shortcut
    pub fn add_project(&mut self, alias: String, path: String) -> Result<()> {
        self.projects.insert(alias, path);
        self.save()
    }

    /// Get the path for a project alias
    pub fn get_project(&self, alias: &str) -> Option<&String> {
        self.projects.get(alias)
    }

    /// List all project aliases
    pub fn list_projects(&self) -> Vec<&String> {
        self.projects.keys().collect()
    }

    /// Standard config path: ~/.config/todo-manager/config.json
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("todo-manager")
            .join("config.json")
    }
}
