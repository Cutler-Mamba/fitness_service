use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContextStore {
    tokens: HashMap<String, String>,
    poll_cursor: Option<String>,
}

impl Default for ContextStore {
    fn default() -> Self {
        Self {
            tokens: HashMap::new(),
            poll_cursor: None,
        }
    }
}

pub struct IlinkSession {
    store: RwLock<ContextStore>,
    file_path: PathBuf,
}

impl IlinkSession {
    pub fn new(context_dir: &str, account_id: &str) -> Self {
        let dir = Path::new(context_dir);
        fs::create_dir_all(dir).ok();

        let file_path = dir.join(format!("{}.context-tokens.json", account_id));
        let store = Self::load_from_disk(&file_path);

        Self {
            store: RwLock::new(store),
            file_path,
        }
    }

    fn load_from_disk(path: &Path) -> ContextStore {
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<ContextStore>(&content) {
                Ok(store) => {
                    debug!("Loaded {} context tokens from disk", store.tokens.len());
                    store
                }
                Err(e) => {
                    warn!("Failed to parse context store: {}, starting fresh", e);
                    ContextStore::default()
                }
            },
            Err(_) => {
                debug!("No existing context store, starting fresh");
                ContextStore::default()
            }
        }
    }

    fn save_to_disk(&self) {
        if let Ok(store) = self.store.read() {
            if let Ok(json) = serde_json::to_string_pretty(&*store) {
                if let Err(e) = fs::write(&self.file_path, &json) {
                    warn!("Failed to save context store: {}", e);
                }
            }
        }
    }

    pub fn get_context_token(&self, user_id: &str) -> Option<String> {
        self.store
            .read()
            .ok()
            .and_then(|s| s.tokens.get(user_id).cloned())
    }

    pub fn set_context_token(&self, user_id: &str, token: String) {
        if let Ok(mut store) = self.store.write() {
            store.tokens.insert(user_id.to_string(), token);
        }
        self.save_to_disk();
    }

    pub fn get_poll_cursor(&self) -> Option<String> {
        self.store.read().ok().and_then(|s| s.poll_cursor.clone())
    }

    pub fn set_poll_cursor(&self, cursor: String) {
        if let Ok(mut store) = self.store.write() {
            store.poll_cursor = Some(cursor);
        }
        self.save_to_disk();
    }

    pub fn token_count(&self) -> usize {
        self.store
            .read()
            .ok()
            .map(|s| s.tokens.len())
            .unwrap_or(0)
    }
}
