use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::path::{Path, PathBuf};
use std::fs;
use crate::kriya::schema::SchemaType;
use crate::store;

pub type ActionHandler = Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>;

pub struct RegisteredAction {
    pub name: String,
    pub description: String,
    pub schema: SchemaType,
    pub handler: ActionHandler,
}

pub struct ActionRegistry {
    pub actions: HashMap<String, RegisteredAction>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, action: RegisteredAction) {
        self.actions.insert(action.name.clone(), action);
    }

    pub fn execute(&self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, String> {
        let action = self.actions.get(name).ok_or_else(|| format!("Action '{}' not found in registry", name))?;
        
        // Validate arguments against the schema
        action.schema.validate(&arguments).map_err(|e| format!("Validation error for action '{}': {}", name, e))?;

        // Execute action handler
        (action.handler)(arguments)
    }
}

// Global lazy or thread-safe instance of the registry
pub static REGISTRY: OnceLock<Mutex<ActionRegistry>> = OnceLock::new();

pub fn get_registry() -> &'static Mutex<ActionRegistry> {
    REGISTRY.get_or_init(|| {
        let mut r = ActionRegistry::new();
        register_default_actions(&mut r);
        Mutex::new(r)
    })
}

fn secure_resolve_path(vault_path: &str, relative_path: &str) -> Result<PathBuf, String> {
    let vault_path_buf = dunce::canonicalize(Path::new(vault_path))
        .map_err(|e| format!("Failed to canonicalize vault path: {}", e))?;
    
    let resolved = Path::new(vault_path).join(relative_path);
    
    // Resolve absolute representation without escaping the vault
    let canonical_target = if resolved.exists() {
        dunce::canonicalize(&resolved)
            .map_err(|e| format!("Failed to canonicalize target path: {}", e))?
    } else {
        let parent = resolved.parent().ok_or("Invalid path: no parent directory")?;
        if parent.exists() {
            dunce::canonicalize(parent)
                .map_err(|e| format!("Failed to canonicalize parent path: {}", e))?
                .join(resolved.file_name().ok_or("Invalid filename")?)
        } else {
            // Fallback to syntactic check if parent doesn't exist
            resolved
        }
    };

    if canonical_target.starts_with(&vault_path_buf) {
        Ok(canonical_target)
    } else {
        Err("Security violation: Path escapes vault boundary".to_string())
    }
}

fn register_default_actions(registry: &mut ActionRegistry) {
    // 1. read_note action
    let read_note_schema = SchemaType::Object {
        properties: {
            let mut props = HashMap::new();
            props.insert("path".to_string(), SchemaType::Str);
            props
        },
        required: vec!["path".to_string()],
    };
    
    registry.register(RegisteredAction {
        name: "read_note".to_string(),
        description: "Reads the content of a markdown note from the active vault.".to_string(),
        schema: read_note_schema,
        handler: Arc::new(|args| {
            let relative_path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing path parameter")?;
            let vault = store::get_active_vault().ok_or("No active vault selected")?;
            
            let target_path = secure_resolve_path(&vault.path, relative_path)?;
            let content = fs::read_to_string(target_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            
            Ok(serde_json::json!({ "content": content }))
        }),
    });

    // 2. write_note action
    let write_note_schema = SchemaType::Object {
        properties: {
            let mut props = HashMap::new();
            props.insert("path".to_string(), SchemaType::Str);
            props.insert("content".to_string(), SchemaType::Str);
            props
        },
        required: vec!["path".to_string(), "content".to_string()],
    };

    registry.register(RegisteredAction {
        name: "write_note".to_string(),
        description: "Creates or overwrites a markdown note in the active vault with new content.".to_string(),
        schema: write_note_schema,
        handler: Arc::new(|args| {
            let relative_path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing path parameter")?;
            let content = args.get("content").and_then(|v| v.as_str()).ok_or("Missing content parameter")?;
            let vault = store::get_active_vault().ok_or("No active vault selected")?;

            let target_path = secure_resolve_path(&vault.path, relative_path)?;
            
            // Create parent directories if they do not exist
            if let Some(parent) = target_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            fs::write(target_path, content)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            Ok(serde_json::json!({ "success": true }))
        }),
    });

    // 3. list_notes action
    let list_notes_schema = SchemaType::Object {
        properties: {
            let mut props = HashMap::new();
            props.insert("folder".to_string(), SchemaType::Optional {
                inner: Box::new(SchemaType::Str),
            });
            props
        },
        required: vec![],
    };

    registry.register(RegisteredAction {
        name: "list_notes".to_string(),
        description: "Lists all markdown notes inside a specific folder of the active vault, or the vault root if omitted.".to_string(),
        schema: list_notes_schema,
        handler: Arc::new(|args| {
            let folder = args.get("folder").and_then(|v| v.as_str()).unwrap_or("");
            let vault = store::get_active_vault().ok_or("No active vault selected")?;

            let target_dir = secure_resolve_path(&vault.path, folder)?;
            if !target_dir.is_dir() {
                return Err(format!("Path is not a directory: {:?}", target_dir));
            }

            let entries = fs::read_dir(target_dir)
                .map_err(|e| format!("Failed to read directory: {}", e))?;

            let mut notes = Vec::new();
            for entry in entries.filter_map(|e| e.ok()) {
                let file_type = entry.file_type().map_err(|e| e.to_string())?;
                if file_type.is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "md" {
                            if let Some(name) = entry.file_name().to_str() {
                                notes.push(name.to_string());
                            }
                        }
                    }
                }
            }

            Ok(serde_json::json!({ "notes": notes }))
        }),
    });
}
