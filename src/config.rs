//! Configuration management for Bear CLI
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub gateway: String,
}

impl Config {
    pub fn new(gateway: String) -> Self {
        Self { gateway }
    }
    
    pub fn default_config() -> Self {
        let gateway = var("BEAR_GATEWAY").unwrap_or_else(|_| "bear-way.ai.studio".to_string());
        Self { gateway }
    }
}

/// Get the path to the config file
pub fn config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Cannot find home directory"))?;
    let config_dir = std::path::Path::new(&home).join(".bear");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir.join("config.toml"))
}

/// Load config from disk, or return default if not exists
pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default_config());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// Save config to disk
pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}