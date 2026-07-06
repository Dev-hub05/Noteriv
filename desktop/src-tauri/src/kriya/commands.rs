use tauri::{AppHandle, Emitter, State};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::kriya::types::{AgentStartRequest, AgentActionResult, StepDecision, AgentActionRequest, Receipt};
use chrono::Utc;
use rand::Rng;
use tokio::sync::oneshot;

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

/// Pending dispatch results — maps request_id → oneshot sender.
/// When the React dispatcher returns a result, we route it to the waiting sender.
pub struct KriyaDispatchState {
    pub pending: Mutex<HashMap<String, oneshot::Sender<AgentActionResult>>>,
}

impl Default for KriyaDispatchState {
    fn default() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }
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
            action_name: "mock_action".to_string(),
            arguments: serde_json::json!({}),
            result: res,
            signature: None,
        };
        session.history.push(receipt);
    }

    // Mock agent execution loop decision logic
    if session.step_count == 1 {
        Ok(StepDecision {
            thought: format!("I need to analyze the user instruction: '{}'. Let's first read a note.", session.initial_instruction),
            action: Some(AgentActionRequest {
                action_name: "read_note".to_string(),
                arguments: serde_json::json!({ "path": "welcome.md" }),
            }),
            final_answer: None,
        })
    } else {
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

/// Dispatches an action to the React frontend for execution.
///
/// Flow:
///   1. Validate args against schema in Rust registry
///   2. Generate a unique request_id
///   3. Create a oneshot channel and store the sender in KriyaDispatchState
///   4. Emit "kriya:dispatch-action" event to React with { request_id, action_name, arguments }
///   5. Await the oneshot receiver (React will call kriya_dispatch_result when done)
///   6. Return the result
#[tauri::command]
pub async fn kriya_execute_action(
    app: AppHandle,
    dispatch_state: State<'_, KriyaDispatchState>,
    action_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    // Step 1: Validate arguments against schema
    {
        let registry = crate::kriya::action_registry::get_registry();
        let reg_lock = registry.lock().map_err(|e| e.to_string())?;
        reg_lock.validate_args(&action_name, &arguments)?;
    }

    // Step 2: Generate request_id
    let request_id = format!("req_{}", rand::thread_rng().gen::<u64>());

    // Step 3: Create oneshot channel
    let (tx, rx) = oneshot::channel::<AgentActionResult>();
    {
        let mut pending = dispatch_state.pending.lock().map_err(|e| e.to_string())?;
        pending.insert(request_id.clone(), tx);
    }

    // Step 4: Emit dispatch event to React
    log::info!("Kriya dispatching action '{}' (request_id: {})", action_name, request_id);
    app.emit("kriya:dispatch-action", serde_json::json!({
        "request_id": request_id,
        "action_name": action_name,
        "arguments": arguments,
    })).map_err(|e| format!("Failed to emit dispatch event: {}", e))?;

    // Step 5: Await result from React
    let result = rx.await.map_err(|_| "Dispatch channel closed — React handler did not respond".to_string())?;

    // Step 6: Return
    Ok(serde_json::json!({
        "status": result.status,
        "output": result.output,
        "error_message": result.error_message,
    }))
}

/// Called by React dispatcher to return the result of an executed action.
#[tauri::command]
pub fn kriya_dispatch_result(
    dispatch_state: State<'_, KriyaDispatchState>,
    request_id: String,
    result: AgentActionResult,
) -> Result<(), String> {
    let mut pending = dispatch_state.pending.lock().map_err(|e| e.to_string())?;
    if let Some(sender) = pending.remove(&request_id) {
        let _ = sender.send(result);
        log::info!("Kriya dispatch result received for request_id: {}", request_id);
        Ok(())
    } else {
        Err(format!("No pending dispatch found for request_id: {}", request_id))
    }
}

#[tauri::command]
pub fn kriya_register_action_metadata(
    metadata: crate::kriya::action_registry::RegisteredActionMetadata,
) -> Result<(), String> {
    let registry = crate::kriya::action_registry::get_registry();
    let mut reg_lock = registry.lock().map_err(|e| e.to_string())?;
    reg_lock.register(metadata);
    Ok(())
}
