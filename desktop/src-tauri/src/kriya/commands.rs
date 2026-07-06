use tauri::State;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::kriya::types::{AgentStartRequest, AgentActionResult, StepDecision, AgentActionRequest, Receipt};
use chrono::Utc;
use rand::Rng;

pub struct KriyaSession {
    pub id: String,
    pub vault_path: String,
    pub initial_instruction: String,
    pub model: Option<String>,
    pub history: Vec<Receipt>,
    pub step_count: u32,
}

#[derive(Default)]
pub struct KriyaState {
    pub sessions: Mutex<HashMap<String, KriyaSession>>,
}

#[tauri::command]
pub fn kriya_start_session(
    state: State<'_, KriyaState>,
    request: AgentStartRequest,
) -> Result<String, String> {
    let session_id = format!("sess_{}", rand::thread_rng().gen::<u32>());
    let session = KriyaSession {
        id: session_id.clone(),
        vault_path: request.vault_path,
        initial_instruction: request.initial_instruction,
        model: request.model,
        history: Vec::new(),
        step_count: 0,
    };

    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.insert(session_id.clone(), session);
    
    log::info!("Started Kriya agent session: {}", session_id);
    Ok(session_id)
}

#[tauri::command]
pub fn kriya_submit_step(
    state: State<'_, KriyaState>,
    session_id: String,
    result: Option<AgentActionResult>,
) -> Result<StepDecision, String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get_mut(&session_id).ok_or_else(|| "Session not found".to_string())?;

    session.step_count += 1;

    // If an action result is provided, record it in history/receipt
    if let Some(res) = result {
        let receipt = Receipt {
            receipt_id: format!("rec_{}", rand::thread_rng().gen::<u32>()),
            timestamp: Utc::now().timestamp_millis() as u64,
            action_name: "mock_action".to_string(), // In a real system, this would trace the requested action name
            arguments: serde_json::json!({}),
            result: res,
            signature: None,
        };
        session.history.push(receipt);
    }

    // Mock agent execution loop decision logic
    if session.step_count == 1 {
        // Step 1: Propose an action to read a note
        Ok(StepDecision {
            thought: format!("I need to analyze the user instruction: '{}'. Let's first read a note.", session.initial_instruction),
            action: Some(AgentActionRequest {
                action_name: "read_note".to_string(),
                arguments: serde_json::json!({ "path": "welcome.md" }),
            }),
            final_answer: None,
        })
    } else {
        // Step 2: Propose final answer based on the result
        Ok(StepDecision {
            thought: "I have read the note. The operation was successful. Exiting loop.".to_string(),
            action: None,
            final_answer: Some(format!(
                "Hello! I completed your request. Initial instruction: '{}'. Checked welcome.md.",
                session.initial_instruction
            )),
        })
    }
}

#[tauri::command]
pub fn kriya_get_session_status(
    state: State<'_, KriyaState>,
    session_id: String,
) -> Result<serde_json::Value, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get(&session_id).ok_or_else(|| "Session not found".to_string())?;

    Ok(serde_json::json!({
        "session_id": session.id,
        "vault_path": session.vault_path,
        "initial_instruction": session.initial_instruction,
        "model": session.model,
        "step_count": session.step_count,
        "history_len": session.history.len(),
    }))
}
