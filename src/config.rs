use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Returns the cross-platform config directory for KeySlop.
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("keyslop");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

/// Returns the path to the config JSON file.
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Returns the default sounds directory inside the config folder.
pub fn default_sounds_dir() -> PathBuf {
    let dir = config_dir().join("sounds");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Optional global sound file path — plays for any key without a specific mapping.
    #[serde(default)]
    pub global_sound: Option<String>,

    /// Per-key sound mappings. Key name (e.g. "KeyA", "Space") -> sound file path.
    #[serde(default)]
    pub key_sounds: HashMap<String, String>,

    /// Master volume (0.0 to 1.0).
    #[serde(default = "default_volume")]
    pub volume: f32,

    /// Whether the listener is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Whether the app should start on boot.
    #[serde(default)]
    pub autostart: bool,

    /// Selected audio output device name. None = system default.
    #[serde(default)]
    pub audio_device: Option<String>,

    /// Max simultaneous sounds (polyphony limit).
    #[serde(default = "default_max_polyphony")]
    pub max_polyphony: usize,
}

fn default_volume() -> f32 {
    1.0
}

fn default_enabled() -> bool {
    true
}

fn default_max_polyphony() -> usize {
    5
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_sound: None,
            key_sounds: HashMap::new(),
            volume: default_volume(),
            enabled: default_enabled(),
            autostart: false,
            audio_device: None,
            max_polyphony: default_max_polyphony(),
        }
    }
}

impl AppConfig {
    /// Load config from disk, or return default if not found.
    pub fn load() -> Self {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Get the sound path for a given key name.
    /// Returns per-key sound if mapped, otherwise global sound.
    pub fn sound_for_key(&self, key_name: &str) -> Option<&String> {
        self.key_sounds.get(key_name).or(self.global_sound.as_ref())
    }
}
