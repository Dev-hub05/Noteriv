use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStartRequest {
    pub vault_path: String,
    pub initial_instruction: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionRequest {
    pub action_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionResult {
    pub status: String, // "success", "error", "pending_approval"
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDecision {
    pub thought: String,
    pub action: Option<AgentActionRequest>,
    pub final_answer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: String,
    pub timestamp: u64,
    pub action_name: String,
    pub arguments: serde_json::Value,
    pub result: AgentActionResult,
    pub signature: Option<String>,
}
