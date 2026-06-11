use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    pub enable_notifications: bool,
    #[serde(default = "default_timeout")]
    pub notification_timeout_in_seconds: i32,
    #[serde(default = "default_lower")]
    pub lower_battery_level: u8,
    #[serde(default = "default_upper")]
    pub upper_battery_level: u8,
    #[serde(default)]
    pub full_charge_level: Option<u8>,
}

fn default_timeout() -> i32 { 5 }
fn default_lower() -> u8 { 10 }
fn default_upper() -> u8 { 20 }

impl Default for Config {
    fn default() -> Self {
        Config {
            enable_notifications: false,
            notification_timeout_in_seconds: default_timeout(),
            lower_battery_level: default_lower(),
            upper_battery_level: default_upper(),
            full_charge_level: None,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("aerox_5");
        path.push("config.toml");
        path
    }

    pub fn exists() -> bool {
        Self::config_path().exists()
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(Self::config_path())?;
        Ok(toml::from_str(&content)?)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn autostart_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("autostart");
    path.push("aerox_5.desktop");
    path
}

pub fn is_autostart_enabled() -> bool {
    autostart_path().exists()
}

pub fn set_autostart(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = autostart_path();
    if enabled {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let exe = std::env::current_exe()?;
        let content = include_str!("../assets/autostart.desktop.template")
            .replace("{exe}", &exe.display().to_string());
        std::fs::write(path, content)?;
    } else if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
