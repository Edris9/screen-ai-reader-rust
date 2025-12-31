use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub local_endpoint: String,
    pub local_model: String,
    pub openai_api_key: String,
    pub openai_model: String,
    pub claude_api_key: String,
    pub claude_model: String,
    pub default_prompt: String,
    pub use_local: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_endpoint: "http://localhost:11434".to_string(),
            local_model: "llava".to_string(),
            openai_api_key: String::new(),
            openai_model: "gpt-4o".to_string(),
            claude_api_key: String::new(),
            claude_model: "claude-sonnet-4-20250514".to_string(),
            default_prompt: "Beskriv vad du ser i denna bild. Om det finns text, läs och återge den.".to_string(),
            use_local: true,
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("screen-ai-reader")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Kunde inte skapa config-mapp: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Kunde inte serialisera config: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("Kunde inte spara config: {}", e))?;
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub prompt: String,
    pub response: String,
    pub image_preview: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    pub fn add(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
        // Behåll max 50 entries
        if self.entries.len() > 50 {
            self.entries.remove(0);
        }
    }
}
