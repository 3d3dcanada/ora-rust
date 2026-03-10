//! OrA Application State

use crate::audit::logger::AuditLogger;
use crate::config::Config;
use crate::kernel::Kernel;
use crate::llm::LlmClient;
use crate::security::gates::AstParser;
use crate::security::vault::Vault;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Real-time event broadcast to WebSocket clients
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Task started
    TaskStarted { task_id: String, task: String },
    /// Plan created
    PlanCreated { task_id: String, steps: Vec<String> },
    /// Step started
    StepStarted {
        task_id: String,
        step_number: u32,
        tool: String,
        description: String,
    },
    /// Step completed
    StepCompleted {
        task_id: String,
        step_number: u32,
        success: bool,
        output: String,
    },
    /// Task completed
    TaskCompleted {
        task_id: String,
        success: bool,
        summary: String,
    },
    /// Approval required
    ApprovalRequired {
        approval_id: String,
        operation: String,
        description: String,
    },
    /// Error occurred
    Error {
        task_id: Option<String>,
        message: String,
    },
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub kernel: Arc<RwLock<Kernel>>,
    pub vault: Arc<RwLock<Vault>>,
    pub security_gates: Arc<AstParser>,
    pub audit_logger: Arc<RwLock<AuditLogger>>,
    /// Broadcast channel for real-time events
    pub event_tx: broadcast::Sender<AgentEvent>,
    /// LLM client (shared)
    pub llm: Arc<LlmClient>,
}

impl AppState {
    pub fn new(
        config: Config,
        kernel: Arc<RwLock<Kernel>>,
        vault: Vault,
        security_gates: Arc<AstParser>,
        audit_logger: Arc<RwLock<AuditLogger>>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        // Create LLM client from config
        // Use llm_model if set, otherwise fall back to default_model
        let model = config
            .llm_model
            .as_ref()
            .unwrap_or(&config.default_model)
            .clone();
        let base_url = config
            .llm_base_url
            .as_ref()
            .or(config.api_base_url.as_ref())
            .cloned();

        let llm = Arc::new(
            LlmClient::from_config(
                &config.llm_provider,
                &model,
                config.llm_api_key.clone(),
                base_url,
            )
            .with_max_tokens(config.max_tokens)
            .with_temperature(config.temperature),
        );

        Self {
            config,
            kernel,
            vault: Arc::new(RwLock::new(vault)),
            security_gates,
            audit_logger,
            event_tx,
            llm,
        }
    }

    /// Subscribe to agent events
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// Broadcast an event
    pub fn broadcast(&self, event: AgentEvent) {
        // Ignore send errors (no subscribers)
        let _ = self.event_tx.send(event);
    }
}
