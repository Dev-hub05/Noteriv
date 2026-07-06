use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub session_id: String,
    pub category: MemoryCategory,
    pub content: String,
    pub metadata: serde_json::Value,
    pub timestamp: u64,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryCategory {
    Episodic,   // Short-term: what happened in a session
    Semantic,   // Long-term: learned facts about the vault
    Procedural, // How-to: patterns the agent discovered
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    pub entries: Vec<MemoryEntry>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn store(&mut self, entry: MemoryEntry) {
        self.entries.push(entry);
    }

    pub fn recall(&self, query: &str, category: Option<MemoryCategory>, limit: usize) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&MemoryEntry> = self.entries.iter()
            .filter(|e| {
                if let Some(ref cat) = category {
                    if &e.category != cat { return false; }
                }
                e.content.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        // Sort by relevance_score descending, then by timestamp descending
        results.sort_by(|a, b| {
            b.relevance_score.partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.timestamp.cmp(&a.timestamp))
        });
        
        results.truncate(limit);
        results
    }

    pub fn recall_by_session(&self, session_id: &str) -> Vec<&MemoryEntry> {
        self.entries.iter()
            .filter(|e| e.session_id == session_id)
            .collect()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let episodic = self.entries.iter().filter(|e| e.category == MemoryCategory::Episodic).count();
        let semantic = self.entries.iter().filter(|e| e.category == MemoryCategory::Semantic).count();
        let procedural = self.entries.iter().filter(|e| e.category == MemoryCategory::Procedural).count();
        
        serde_json::json!({
            "total": self.entries.len(),
            "episodic": episodic,
            "semantic": semantic,
            "procedural": procedural,
        })
    }
}

pub struct PersistentMemory {
    store: Mutex<MemoryStore>,
    file_path: Mutex<Option<PathBuf>>,
}

impl PersistentMemory {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(MemoryStore::new()),
            file_path: Mutex::new(None),
        }
    }

    pub fn initialize(&self, vault_path: &str) {
        let memory_dir = Path::new(vault_path).join(".noteriv").join("kriya");
        let _ = std::fs::create_dir_all(&memory_dir);
        let path = memory_dir.join("memory.json");

        // Load existing memory if file exists
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(store) = serde_json::from_str::<MemoryStore>(&data) {
                    *self.store.lock().unwrap() = store;
                    log::info!("Kriya memory loaded from {:?}", path);
                }
            }
        }

        *self.file_path.lock().unwrap() = Some(path);
    }

    fn persist(&self) {
        let file_path = self.file_path.lock().unwrap();
        if let Some(ref path) = *file_path {
            let store = self.store.lock().unwrap();
            if let Ok(json) = serde_json::to_string_pretty(&*store) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    pub fn store_entry(&self, session_id: &str, category: MemoryCategory, content: String, metadata: serde_json::Value, relevance: f64) -> String {
        let id = format!("mem_{}", rand::random::<u64>());
        let entry = MemoryEntry {
            id: id.clone(),
            session_id: session_id.to_string(),
            category,
            content,
            metadata,
            timestamp: Utc::now().timestamp_millis() as u64,
            relevance_score: relevance,
        };
        self.store.lock().unwrap().store(entry);
        self.persist();
        id
    }

    pub fn recall(&self, query: &str, category: Option<MemoryCategory>, limit: usize) -> Vec<MemoryEntry> {
        let store = self.store.lock().unwrap();
        store.recall(query, category, limit).into_iter().cloned().collect()
    }

    pub fn recall_by_session(&self, session_id: &str) -> Vec<MemoryEntry> {
        let store = self.store.lock().unwrap();
        store.recall_by_session(session_id).into_iter().cloned().collect()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        self.store.lock().unwrap().get_stats()
    }
}

impl Default for PersistentMemory {
    fn default() -> Self {
        Self::new()
    }
}

static MEMORY: std::sync::OnceLock<PersistentMemory> = std::sync::OnceLock::new();

pub fn get_memory() -> &'static PersistentMemory {
    MEMORY.get_or_init(PersistentMemory::new)
}
