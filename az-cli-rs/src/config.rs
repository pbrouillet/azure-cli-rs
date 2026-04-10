/// Config system — reads/writes `~/.azure/config` (INI format, same as az Python).
///
/// Sections:
///   [core]     — output (default output format)
///   [defaults] — group, location
use crate::error::{AzrsError, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Parsed config file — section → key → value.
#[derive(Debug, Default)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    fn path() -> PathBuf {
        let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push(".azure");
        p.push("config");
        p
    }

    /// Load config from disk (or empty if missing).
    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => Self::parse(&content),
            Err(_) => Self::default(),
        }
    }

    /// Parse an INI-format string.
    fn parse(content: &str) -> Self {
        let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut current_section = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                sections
                    .entry(current_section.clone())
                    .or_default()
                    .insert(
                        key.trim().to_string(),
                        value.trim().to_string(),
                    );
            }
        }

        Self { sections }
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut output = String::new();
        // Sort sections for stable output
        let mut section_names: Vec<&String> = self.sections.keys().collect();
        section_names.sort();

        for section in section_names {
            if let Some(entries) = self.sections.get(section) {
                if !section.is_empty() {
                    output.push_str(&format!("[{section}]\n"));
                }
                let mut keys: Vec<&String> = entries.keys().collect();
                keys.sort();
                for key in keys {
                    if let Some(val) = entries.get(key) {
                        output.push_str(&format!("{key} = {val}\n"));
                    }
                }
                output.push('\n');
            }
        }

        std::fs::write(&path, output)?;
        Ok(())
    }

    /// Get a value: `config.get("defaults", "group")`.
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|s| s.get(key))
            .map(|s| s.as_str())
    }

    /// Set a value: `config.set("defaults", "group", "MyRG")`.
    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    /// Unset a value.
    pub fn unset(&mut self, section: &str, key: &str) {
        if let Some(s) = self.sections.get_mut(section) {
            s.remove(key);
        }
    }

    // --- Convenience accessors ---

    /// Default resource group from [defaults] group.
    #[allow(dead_code)]
    pub fn default_group(&self) -> Option<&str> {
        self.get("defaults", "group")
    }

    /// Default location from [defaults] location.
    #[allow(dead_code)]
    pub fn default_location(&self) -> Option<&str> {
        self.get("defaults", "location")
    }

    /// Default output format from [core] output.
    #[allow(dead_code)]
    pub fn default_output(&self) -> Option<&str> {
        self.get("core", "output")
    }
}

// --- CLI commands ---

/// `azrs config set <key=value>...`
/// Keys use dot notation: `defaults.group=MyRG`, `core.output=table`
pub fn config_set(pairs: &[String]) -> Result<()> {
    let mut config = Config::load();
    for pair in pairs {
        let (key, value) = pair.split_once('=').ok_or_else(|| {
            AzrsError::General(format!("Invalid format '{pair}'. Expected section.key=value"))
        })?;
        let (section, prop) = key.split_once('.').ok_or_else(|| {
            AzrsError::General(format!("Invalid key '{key}'. Expected section.key (e.g. defaults.group)"))
        })?;
        config.set(section, prop, value);
    }
    config.save()?;
    Ok(())
}

/// `azrs config get <key>`
pub fn config_get(key: &str) -> Result<Option<serde_json::Value>> {
    let config = Config::load();
    let (section, prop) = key.split_once('.').ok_or_else(|| {
        AzrsError::General(format!("Invalid key '{key}'. Expected section.key (e.g. defaults.group)"))
    })?;
    match config.get(section, prop) {
        Some(val) => Ok(Some(serde_json::Value::String(val.to_string()))),
        None => Ok(None),
    }
}

/// `azrs config unset <key>`
pub fn config_unset(key: &str) -> Result<()> {
    let mut config = Config::load();
    let (section, prop) = key.split_once('.').ok_or_else(|| {
        AzrsError::General(format!("Invalid key '{key}'. Expected section.key"))
    })?;
    config.unset(section, prop);
    config.save()?;
    Ok(())
}
