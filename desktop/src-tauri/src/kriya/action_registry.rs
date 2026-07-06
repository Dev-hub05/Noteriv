use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::kriya::schema::SchemaType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredActionMetadata {
    pub name: String,
    pub description: String,
    pub schema: SchemaType,
}

#[derive(Default)]
pub struct ActionRegistry {
    pub actions: HashMap<String, RegisteredActionMetadata>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, action: RegisteredActionMetadata) {
        log::info!("Kriya registered action metadata: {}", action.name);
        self.actions.insert(action.name.clone(), action);
    }

    pub fn validate_args(&self, name: &str, arguments: &serde_json::Value) -> Result<(), String> {
        let action = self.actions.get(name).ok_or_else(|| format!("Action '{}' not found in registry", name))?;
        action.schema.validate(arguments).map_err(|e| format!("Validation error for action '{}': {}", name, e))
    }
}

// Global thread-safe instance of the metadata registry
pub static REGISTRY: OnceLock<Mutex<ActionRegistry>> = OnceLock::new();

pub fn get_registry() -> &'static Mutex<ActionRegistry> {
    REGISTRY.get_or_init(|| Mutex::new(ActionRegistry::new()))
}
