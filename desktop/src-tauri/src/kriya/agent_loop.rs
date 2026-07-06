use tauri::{AppHandle, Emitter, State};
use crate::kriya::types::{AgentStartRequest, AgentActionResult, StepDecision, AgentActionRequest, Receipt};
use crate::kriya::commands::{KriyaState, KriyaDispatchState, execute_action_internal};
use crate::kriya::inference::{InferenceBackend, InferenceConfig, ChatMessage};
use crate::kriya::action_registry;
use chrono::Utc;
use rand::Rng;

fn get_vault_context(vault_path: &str) -> String {
    let mut files_count = 0;
    let mut dirs_count = 0;
    
    if let Ok(entries) = std::fs::read_dir(vault_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(ft) = entry.file_type() {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.') {
                        if ft.is_dir() {
                            dirs_count += 1;
                        } else {
                            files_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    format!(
        "Vault Path: {}\nTop-level active files: {}\nTop-level active folders: {}\n",
        vault_path, files_count, dirs_count
    )
}

fn build_system_prompt(vault_path: &str) -> String {
    let registry = action_registry::get_registry();
    let reg_lock = registry.lock().unwrap();
    
    // Format registered actions list
    let mut actions_desc = String::new();
    for action in reg_lock.actions.values() {
        actions_desc.push_str(&format!(
            "- name: {}\n  description: {}\n  schema: {}\n\n",
            action.name,
            action.description,
            serde_json::to_string_pretty(&action.schema).unwrap_or_default()
        ));
    }

    let vault_context = get_vault_context(vault_path);

    format!(
        "You are an AI Agent operating inside Noteriv, a desktop markdown note editor. \
        You help the user manage, read, write, and organize their note vault.\n\n\
        VAULT CONTEXT:\n\
        {}\n\n\
        You must operate in a step-by-step loop. In each step, you can choose to execute one of the registered actions below, or provide a final answer to the user if you have completed the request.\n\n\
        AVAILABLE ACTIONS:\n\
        {}\n\n\
        RESPONSE FORMAT:\n\
        You MUST respond ONLY with a single valid JSON block conforming to the following structure. Do not output any thinking or text outside of this JSON block:\n\
        {{\n  \
          \"thought\": \"Detailed description of your thought process, what you are planning, and why.\",\n  \
          \"action\": {{\n    \
            \"action_name\": \"Name of the action to execute (must be one of the names listed above)\",\n    \
            \"arguments\": {{ ... }} // Arguments matching the action's schema\n  \
          }},\n  \
          \"final_answer\": \"Your final message/response to the user if the task is completely finished. Set to null if you are requesting an action.\"\n\
        }}\n\n\
        RULES:\n\
        1. If you need to perform an action, provide the 'action' block and set 'final_answer' to null.\n\
        2. Once the task is fully complete, set 'action' to null and provide a detailed explanation in 'final_answer'.\n\
        3. Never output text outside of the JSON block.",
        vault_context,
        actions_desc
    )
}

fn build_messages(
    initial_instruction: &str,
    history: &[Receipt],
    system_prompt: &str,
) -> Vec<ChatMessage> {
    let mut messages = Vec::new();
    
    // 1. System Prompt
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    });

    // 2. User Initial Instruction
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: format!("User Request: {}", initial_instruction),
    });

    // 3. Step history (Actions and results)
    for (i, receipt) in history.iter().enumerate() {
        // Mock assistant thought/decision
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: serde_json::json!({
                "thought": format!("Step {} execution: Calling {}", i + 1, receipt.action_name),
                "action": {
                    "action_name": receipt.action_name,
                    "arguments": receipt.arguments
                },
                "final_answer": null
            }).to_string(),
        });

        // System result feed
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Action Result for '{}': status: {}, output: {}, error_message: {}",
                receipt.action_name,
                receipt.result.status,
                serde_json::to_string(&receipt.result.output).unwrap_or_default(),
                receipt.result.error_message.as_deref().unwrap_or("none")
            ),
        });
    }

    messages
}

pub fn run_agent_loop(
    app: AppHandle,
    state: State<'_, KriyaState>,
    dispatch_state: State<'_, KriyaDispatchState>,
    session_id: String,
    backend_config: InferenceConfig,
) {
    let backend = InferenceBackend::new(backend_config);
    let state_clone = app.state::<KriyaState>();
    let dispatch_clone = app.state::<KriyaDispatchState>();

    tokio::spawn(async move {
        let max_steps = 10;
        let mut step_count = 0;

        log::info!("Kriya Agent Loop started for session {}", session_id);

        loop {
            step_count += 1;
            if step_count > max_steps {
                log::warn!("Kriya session {} exceeded maximum steps limit.", session_id);
                let _ = app.emit("kriya:agent-finished", serde_json::json!({
                    "session_id": session_id,
                    "error": "Execution step limit exceeded"
                }));
                break;
            }

            // 1. Retrieve current session state
            let (initial_instruction, history, vault_path) = {
                let sessions = state_clone.sessions.lock().unwrap();
                match sessions.get(&session_id) {
                    Some(sess) => (sess.initial_instruction.clone(), sess.history.clone(), sess.vault_path.clone()),
                    None => {
                        log::error!("Kriya session {} not found in state.", session_id);
                        break;
                    }
                }
            };

            // 2. Format context
            let system_prompt = build_system_prompt(&vault_path);
            let messages = build_messages(&initial_instruction, &history, &system_prompt);

            // 3. Query Inference Backend
            let decision = match backend.get_step_decision(&messages).await {
                Ok(dec) => dec,
                Err(e) => {
                    log::error!("Kriya agent loop backend error: {}", e);
                    let _ = app.emit("kriya:agent-finished", serde_json::json!({
                        "session_id": session_id,
                        "error": format!("Inference error: {}", e)
                    }));
                    break;
                }
            };

            // 4. Emit thought to React frontend
            let _ = app.emit("kriya:agent-thought", serde_json::json!({
                "session_id": session_id,
                "thought": decision.thought,
            }));

            // 5. Handle Final Answer
            if let Some(final_ans) = decision.final_answer {
                log::info!("Kriya session {} finished successfully with final answer.", session_id);
                let _ = app.emit("kriya:agent-finished", serde_json::json!({
                    "session_id": session_id,
                    "final_answer": final_ans
                }));
                break;
            }

            // 6. Handle Action Request
            if let Some(action_req) = decision.action {
                log::info!("Kriya agent session {} executing action: {}", session_id, action_req.action_name);
                
                let _ = app.emit("kriya:agent-action-start", serde_json::json!({
                    "session_id": session_id,
                    "action_name": action_req.action_name
                }));

                // Execute action using our React-dispatching pipeline!
                let val_res = execute_action_internal(
                    app.clone(),
                    &dispatch_clone,
                    Some(session_id.clone()),
                    action_req.action_name.clone(),
                    action_req.arguments.clone(),
                ).await;

                let signature = match &val_res {
                    Ok(val) => val.get("signature").and_then(|s| s.as_str()).map(|s| s.to_string()),
                    Err(_) => None,
                };

                let action_res = match val_res {
                    Ok(val) => AgentActionResult {
                        status: val.get("status").and_then(|s| s.as_str()).unwrap_or("success").to_string(),
                        output: val.get("output").cloned(),
                        error_message: val.get("error_message").and_then(|s| s.as_str()).map(|s| s.to_string()),
                    },
                    Err(err) => AgentActionResult {
                        status: "error".to_string(),
                        output: None,
                        error_message: Some(err),
                    },
                };

                // Add receipt to session history
                {
                    let mut sessions = state_clone.sessions.lock().unwrap();
                    if let Some(sess) = sessions.get_mut(&session_id) {
                        let receipt = Receipt {
                            receipt_id: format!("rec_{}", rand::thread_rng().gen::<u64>()),
                            timestamp: Utc::now().timestamp_millis() as u64,
                            action_name: action_req.action_name,
                            arguments: action_req.arguments,
                            result: action_res,
                            signature,
                        };
                        sess.history.push(receipt);
                    }
                }
            } else {
                log::error!("Kriya agent loop received empty action and final answer.");
                let _ = app.emit("kriya:agent-finished", serde_json::json!({
                    "session_id": session_id,
                    "error": "Invalid LLM decision output: missing both action and final answer."
                }));
                break;
            }
        }
    });
}
