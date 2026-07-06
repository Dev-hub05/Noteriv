use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SchemaType {
    Str,
    Num,
    Bool,
    Array {
        items: Box<SchemaType>,
    },
    Enum {
        values: Vec<String>,
    },
    Object {
        properties: HashMap<String, SchemaType>,
        required: Vec<String>,
    },
    Optional {
        inner: Box<SchemaType>,
    },
}

impl SchemaType {
    pub fn validate(&self, value: &serde_json::Value) -> Result<(), String> {
        match self {
            SchemaType::Str => {
                if value.is_string() {
                    Ok(())
                } else {
                    Err(format!("Expected string, found: {:?}", value))
                }
            }
            SchemaType::Num => {
                if value.is_number() {
                    Ok(())
                } else {
                    Err(format!("Expected number, found: {:?}", value))
                }
            }
            SchemaType::Bool => {
                if value.is_boolean() {
                    Ok(())
                } else {
                    Err(format!("Expected boolean, found: {:?}", value))
                }
            }
            SchemaType::Array { items } => {
                if let Some(arr) = value.as_array() {
                    for (i, val) in arr.iter().enumerate() {
                        items.validate(val).map_err(|e| format!("Array index {}: {}", i, e))?;
                    }
                    Ok(())
                } else {
                    Err(format!("Expected array, found: {:?}", value))
                }
            }
            SchemaType::Enum { values } => {
                if let Some(s) = value.as_str() {
                    if values.contains(&s.to_string()) {
                        Ok(())
                    } else {
                        Err(format!("Value '{}' not in allowed enum list {:?}", s, values))
                    }
                } else {
                    Err(format!("Expected enum string, found: {:?}", value))
                }
            }
            SchemaType::Object { properties, required } => {
                if let Some(obj) = value.as_object() {
                    for req in required {
                        if !obj.contains_key(req) {
                            return Err(format!("Missing required object property: '{}'", req));
                        }
                    }
                    for (key, val) in obj {
                        if let Some(prop_schema) = properties.get(key) {
                            prop_schema.validate(val).map_err(|e| format!("Property '{}': {}", key, e))?;
                        }
                    }
                    Ok(())
                } else {
                    Err(format!("Expected object, found: {:?}", value))
                }
            }
            SchemaType::Optional { inner } => {
                if value.is_null() {
                    Ok(())
                } else {
                    inner.validate(value)
                }
            }
        }
    }
}
