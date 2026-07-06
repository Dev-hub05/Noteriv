use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::kriya::types::StepDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InferenceConfig {
    Mock,
    Ollama { model: String, host: Option<String> },
    Anthropic { api_key: String, model: Option<String> },
}

pub struct InferenceBackend {
    client: reqwest::Client,
    config: InferenceConfig,
}

impl InferenceBackend {
    pub fn new(config: InferenceConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { client, config }
    }

    pub async fn chat_completion(&self, messages: &[ChatMessage]) -> Result<String, String> {
        match &self.config {
            InferenceConfig::Mock => {
                // Returns mock decision content
                Ok(r#"{
                    "thought": "Let's read the welcome.md note first.",
                    "action": {
                        "action_name": "read_note",
                        "arguments": { "path": "welcome.md" }
                    },
                    "final_answer": null
                }"#.to_string())
            }
            InferenceConfig::Ollama { model, host } => {
                let url = format!("{}/api/chat", host.as_deref().unwrap_or("http://localhost:11434"));
                
                let body = serde_json::json!({
                    "model": model,
                    "messages": messages,
                    "stream": false,
                    "format": "json",
                    "options": {
                        "temperature": 0.0
                    }
                });

                let res = self.client.post(&url)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("Ollama request failed: {}", e))?;

                if !res.status().is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(format!("Ollama error (status {}): {}", res.status(), err_text));
                }

                let response_json: serde_json::Value = res.json()
                    .await
                    .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

                let content = response_json.pointer("/message/content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Ollama response missing message content".to_string())?;

                Ok(content.to_string())
            }
            InferenceConfig::Anthropic { api_key, model } => {
                let url = "https://api.anthropic.com/v1/messages";
                let model_name = model.as_deref().unwrap_or("claude-3-5-sonnet-latest");

                // Split system message from the message history for Anthropic's API format
                let mut system_prompt = String::new();
                let mut anthropic_messages = Vec::new();

                for msg in messages {
                    if msg.role == "system" {
                        system_prompt.push_str(&msg.content);
                        system_prompt.push_str("\n");
                    } else {
                        anthropic_messages.push(serde_json::json!({
                            "role": msg.role,
                            "content": msg.content
                        }));
                    }
                }

                let body = serde_json::json!({
                    "model": model_name,
                    "messages": anthropic_messages,
                    "system": if system_prompt.is_empty() { None } else { Some(system_prompt) },
                    "max_tokens": 4096,
                    "temperature": 0.0
                });

                let res = self.client.post(url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("Anthropic request failed: {}", e))?;

                if !res.status().is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(format!("Anthropic error (status {}): {}", res.status(), err_text));
                }

                let response_json: serde_json::Value = res.json()
                    .await
                    .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

                let content = response_json.pointer("/content/0/text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Anthropic response missing text content".to_string())?;

                Ok(content.to_string())
            }
        }
    }

    pub async fn get_step_decision(&self, messages: &[ChatMessage]) -> Result<StepDecision, String> {
        let raw_response = self.chat_completion(messages).await?;
        
        // Clean markdown code blocks from JSON if returned
        let cleaned = if raw_response.trim().starts_with("```") {
            let without_start = raw_response.trim_start_matches("```json").trim_start_matches("```");
            without_start.trim_end_matches("```").trim().to_string()
        } else {
            raw_response.trim().to_string()
        };

        let decision: StepDecision = serde_json::from_str(&cleaned)
            .map_err(|e| format!("Failed to parse LLM output into StepDecision: {}\nOriginal output: {}", e, raw_response))?;
        
        Ok(decision)
    }
}
