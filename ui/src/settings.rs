//! User settings and the cached library, stored under the platform config directory
//! (`~/.config/pocketbox` on Linux). Set `POCKETBOX_CONFIG_DIR` to use another directory.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::library::Game;

const SETTINGS_FILE: &str = "settings.toml";
const LIBRARY_FILE: &str = "library.json";

/// Persistent user preferences.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Folders scanned for ROMs, in the order the user added them.
    pub folders: Vec<String>,
    /// Descend into subfolders when scanning.
    pub include_subfolders: bool,
    /// Rescan all folders when the app starts.
    pub rescan_on_start: bool,
    /// Whether a scan has completed at least once (controls the library empty state).
    pub scanned: bool,
    /// LCD palette name: `Green` or `Gray`.
    pub lcd_palette: String,
    /// When each ROM (by path) was last launched, as Unix seconds.
    pub last_played: BTreeMap<String, i64>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            folders: Vec::new(),
            include_subfolders: true,
            rescan_on_start: true,
            scanned: false,
            lcd_palette: "Green".to_string(),
            last_played: BTreeMap::new(),
        }
    }
}

/// Directory holding the settings and library cache.
pub fn config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("POCKETBOX_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pocketbox")
}

impl Settings {
    /// Load settings from `dir`, falling back to defaults when missing or unreadable.
    pub fn load(dir: &Path) -> Settings {
        let path = dir.join(SETTINGS_FILE);
        match std::fs::read_to_string(&path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(settings) => {
                    log::info!(target: "Core", "Loaded settings from {}", path.display());
                    settings
                }
                Err(e) => {
                    log::warn!(target: "Core", "Ignoring unreadable {}: {}", path.display(), e);
                    Settings::default()
                }
            },
            Err(_) => Settings::default(),
        }
    }

    /// Save settings into `dir`, creating it if needed.
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let text = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(SETTINGS_FILE), text).map_err(|e| e.to_string())
    }
}

/// Load the games found by the last scan.
pub fn load_library(dir: &Path) -> Vec<Game> {
    std::fs::read_to_string(dir.join(LIBRARY_FILE))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Save the games found by a scan so the library is available on the next start.
pub fn save_library(dir: &Path, games: &[Game]) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string(games).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(LIBRARY_FILE), text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_defaults_missing_fields() {
        let dir = std::env::temp_dir().join(format!("pocketbox-settings-{}", std::process::id()));
        let mut s = Settings::default();
        s.folders.push("/roms".into());
        s.last_played.insert("/roms/a.gb".into(), 42);
        s.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir), s);

        std::fs::write(dir.join(SETTINGS_FILE), "folders = [\"/x\"]\n").unwrap();
        let partial = Settings::load(&dir);
        assert_eq!(partial.folders, vec!["/x".to_string()]);
        assert!(partial.include_subfolders);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
