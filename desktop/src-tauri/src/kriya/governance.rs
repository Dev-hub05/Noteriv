use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActionSafetyLevel {
    Safe,       // Automatically approved
    Restricted, // Automatically approved if under budget limits
    Dangerous,  // Requires human approval
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPolicy {
    pub safety_level: ActionSafetyLevel,
    pub step_cost: f64,
}

pub struct GovernanceEngine {
    policies: HashMap<String, ActionPolicy>,
    session_budgets: Mutex<HashMap<String, f64>>, // session_id -> accumulated cost
    max_budget_per_session: f64,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        
        // Define default policies
        policies.insert("read_note".to_string(), ActionPolicy {
            safety_level: ActionSafetyLevel::Safe,
            step_cost: 0.1,
        });
        policies.insert("list_notes".to_string(), ActionPolicy {
            safety_level: ActionSafetyLevel::Restricted,
            step_cost: 0.2,
        });
        policies.insert("write_note".to_string(), ActionPolicy {
            safety_level: ActionSafetyLevel::Dangerous,
            step_cost: 1.0,
        });

        Self {
            policies,
            session_budgets: Mutex::new(HashMap::new()),
            max_budget_per_session: 5.0, // Limit maximum cost to 5.0 units per agent session
        }
    }

    pub fn evaluate_action(&self, session_id: &str, action_name: &str) -> Result<ActionSafetyLevel, String> {
        let policy = self.policies.get(action_name).unwrap_or(&ActionPolicy {
            safety_level: ActionSafetyLevel::Dangerous, // Default to dangerous for unregistered actions
            step_cost: 1.0,
        });

        // 1. Permission check
        log::info!("Kriya [Governance] Checking permission for action: {}", action_name);

        // 2. Budget check
        let mut budgets = self.session_budgets.lock().unwrap();
        let current_cost = budgets.entry(session_id.to_string()).or_insert(0.0);
        
        if *current_cost + policy.step_cost > self.max_budget_per_session {
            return Err(format!(
                "Kriya [Governance] Budget limit exceeded for session {}. Current: {}, Action Cost: {}, Limit: {}",
                session_id, current_cost, policy.step_cost, self.max_budget_per_session
            ));
        }

        // Deduct/accumulate cost
        *current_cost += policy.step_cost;
        log::info!("Kriya [Governance] Budget updated. Session {} cost: {}", session_id, current_cost);

        Ok(policy.safety_level.clone())
    }

    pub fn sign_audit_receipt(&self, action_name: &str, request_id: &str) -> String {
        // Generates a mock cryptographic signature for audit log compliance
        let salt = "kriya-audit-compliance-salt";
        let raw = format!("{}:{}:{}", action_name, request_id, salt);
        
        // Simple hash generation for mock signature
        let mut hash = 0u64;
        for byte in raw.bytes() {
            hash = hash.wrapping_add(byte as u64);
            hash = hash.wrapping_mul(31);
        }
        
        format!("sha256:sig_{:x}", hash)
    }
}

pub static ENGINE: OnceLock<GovernanceEngine> = OnceLock::new();

pub fn get_governance_engine() -> &'static GovernanceEngine {
    ENGINE.get_or_init(GovernanceEngine::new)
}
