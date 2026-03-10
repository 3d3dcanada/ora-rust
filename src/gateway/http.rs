//! OrA HTTP Gateway
//!
//! REST API routes for OrA

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::OraError;
use crate::state::AgentEvent;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct VaultStatusResponse {
    pub unlocked: bool,
    pub tier: String,
}

#[derive(Debug, Deserialize)]
pub struct VaultUnlockRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct VaultUnlockResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
pub struct SecurityGateStatus {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SecurityStatusResponse {
    pub gates: Vec<SecurityGateStatus>,
}

#[derive(Debug, Serialize)]
pub struct AuthorityResponse {
    pub authority_level: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthorityEscalationRequest {
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ApprovalInfo {
    pub id: String,
    pub agent: String,
    pub operation: String,
}

#[derive(Debug, Serialize)]
pub struct ApprovalsResponse {
    pub approvals: Vec<ApprovalInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalActionRequest {
    pub approver: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApprovalActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
    pub requires_approval: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub command: String,
    pub message: String,
    pub session_id: Option<String>,
}

pub fn create_router(config: Config, state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/config", get(config_handler))
        .route("/vault/status", get(vault_status))
        .route("/vault/unlock", post(vault_unlock))
        .route("/vault/lock", post(vault_lock))
        .route("/router/models", get(models_handler))
        .route("/security/status", get(security_status))
        .route("/authority/current", get(authority_current))
        .route("/authority/escalate", post(authority_escalate))
        .route("/approvals", get(approvals_list))
        .route("/approvals/:id/approve", post(approval_approve))
        .route("/approvals/:id/reject", post(approval_reject))
        .route("/kernel/metrics", get(kernel_metrics))
        .route("/kernel/process", post(kernel_process))
        .route("/chat", post(chat_handler))
        .route("/ws", get(super::websocket::websocket_handler))
        .with_state((config, state))
}

fn ora_system_prompt() -> &'static str {
    r#"You are OrA, an AI assistant with a security-first mindset.
You help users with tasks while being aware of security implications.
Be concise and helpful. If a request seems dangerous, warn the user."#
}

fn permissions_for_level(level: u8) -> Vec<String> {
    let mut permissions = vec!["read".to_string()];
    if level >= 1 {
        permissions.push("write".to_string());
    }
    if level >= 2 {
        permissions.push("execute".to_string());
    }
    if level >= 3 {
        permissions.push("admin".to_string());
    }
    if level >= 4 {
        permissions.push("vault".to_string());
    }
    if level >= 5 {
        permissions.push("kernel".to_string());
    }
    permissions
}

async fn generate_chat_response(state: &AppState, command: &str) -> Result<String, OraError> {
    if !state.llm.is_configured() {
        return Err(OraError::CredentialNotFound {
            provider: state.config.llm_provider.clone(),
        });
    }

    state.llm.chat(ora_system_prompt(), command).await
}

fn map_chat_error(error: OraError) -> String {
    match error {
        OraError::CredentialNotFound { provider } => {
            format!(
                "LLM provider '{}' is not configured. Set credentials or switch providers.",
                provider
            )
        }
        OraError::ModelNotAvailable { model } => {
            format!("Configured model is not available: {}", model)
        }
        other => format!("Error: {}", other),
    }
}

async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "OrA Backend",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn config_handler(State((config, _)): State<(Config, AppState)>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "host": config.host,
        "port": config.port,
        "workspace_root": config.workspace_root.to_string_lossy(),
        "llm_provider": config.llm_provider,
        "default_model": config.default_model,
    }))
}

async fn vault_status(State((_, state)): State<(Config, AppState)>) -> Json<VaultStatusResponse> {
    let vault = state.vault.read().await;
    Json(VaultStatusResponse {
        unlocked: vault.is_unlocked(),
        tier: "TIER_1".to_string(),
    })
}

async fn vault_unlock(
    State((_, state)): State<(Config, AppState)>,
    Json(req): Json<VaultUnlockRequest>,
) -> Json<VaultUnlockResponse> {
    let mut vault = state.vault.write().await;
    let _ = vault.unlock(Some(&req.password));
    Json(VaultUnlockResponse {
        success: true,
        message: "Vault unlocked".to_string(),
    })
}

async fn vault_lock(State((_, state)): State<(Config, AppState)>) -> Json<serde_json::Value> {
    let mut vault = state.vault.write().await;
    vault.lock();
    Json(serde_json::json!({"success": true}))
}

async fn models_handler(State((_, state)): State<(Config, AppState)>) -> Json<ModelsResponse> {
    let models = state
        .llm
        .available_models()
        .await
        .unwrap_or_else(|_| vec![state.llm.fallback_model_info()]);

    Json(ModelsResponse {
        models: models
            .into_iter()
            .map(|model| ModelInfo {
                id: model.id,
                name: model.name,
                provider: model.provider,
                selected: model.selected,
            })
            .collect(),
    })
}

async fn security_status(
    State((config, _)): State<(Config, AppState)>,
) -> Json<SecurityStatusResponse> {
    let status = if config.security_gates_enabled {
        "enabled"
    } else {
        "disabled"
    };
    Json(SecurityStatusResponse {
        gates: vec![
            SecurityGateStatus {
                id: "prompt_injection".into(),
                name: "Prompt Injection".into(),
                status: status.into(),
            },
            SecurityGateStatus {
                id: "shell".into(),
                name: "Shell Sanitizer".into(),
                status: status.into(),
            },
            SecurityGateStatus {
                id: "sandbox".into(),
                name: "Sandbox".into(),
                status: status.into(),
            },
        ],
    })
}

async fn authority_current(
    State((config, _)): State<(Config, AppState)>,
) -> Json<AuthorityResponse> {
    Json(AuthorityResponse {
        authority_level: format!("A{}", config.max_authority_level),
        permissions: permissions_for_level(config.max_authority_level),
    })
}

async fn authority_escalate(
    State((config, _)): State<(Config, AppState)>,
    Json(_req): Json<AuthorityEscalationRequest>,
) -> Json<AuthorityResponse> {
    let escalated_level = (config.max_authority_level + 1).min(5);
    Json(AuthorityResponse {
        authority_level: format!("A{}", escalated_level),
        permissions: permissions_for_level(escalated_level),
    })
}

async fn approvals_list() -> Json<ApprovalsResponse> {
    Json(ApprovalsResponse { approvals: vec![] })
}

async fn approval_approve(
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(_req): Json<ApprovalActionRequest>,
) -> Json<ApprovalActionResponse> {
    Json(ApprovalActionResponse {
        success: true,
        message: format!("Approval {} approved", id),
    })
}

async fn approval_reject(
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(_req): Json<ApprovalActionRequest>,
) -> Json<ApprovalActionResponse> {
    Json(ApprovalActionResponse {
        success: true,
        message: format!("Approval {} rejected", id),
    })
}

async fn kernel_metrics() -> Json<MetricsResponse> {
    Json(MetricsResponse {
        cpu_usage: 0.0,
        memory_usage: 0.0,
    })
}

async fn kernel_process(
    State((_, state)): State<(Config, AppState)>,
    Json(req): Json<ProcessRequest>,
) -> Json<ChatResponse> {
    let task_id = req
        .session_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let command = req.command.clone();

    let gate_result = state.security_gates.parse_prompt(&command);

    let response = if !gate_result.passed {
        let reason = gate_result.reason.clone().unwrap_or_default();
        state.broadcast(AgentEvent::Error {
            task_id: Some(task_id.clone()),
            message: format!("Security blocked: {}", reason),
        });
        format!("Security blocked: {}", reason)
    } else {
        state.broadcast(AgentEvent::TaskStarted {
            task_id: task_id.clone(),
            task: command.clone(),
        });

        match generate_chat_response(&state, &command).await {
            Ok(output) => {
                state.broadcast(AgentEvent::TaskCompleted {
                    task_id: task_id.clone(),
                    success: true,
                    summary: output.clone(),
                });
                output
            }
            Err(error) => {
                let message = map_chat_error(error);
                state.broadcast(AgentEvent::Error {
                    task_id: Some(task_id.clone()),
                    message: message.clone(),
                });
                message
            }
        }
    };

    Json(ChatResponse {
        response,
        session_id: task_id,
        requires_approval: false,
    })
}

async fn chat_handler(
    State(state): State<(Config, AppState)>,
    Json(req): Json<ChatRequest>,
) -> Json<ChatResponse> {
    kernel_process(
        State(state),
        Json(ProcessRequest {
            command: req.message.clone(),
            message: req.message,
            session_id: req.session_id,
        }),
    )
    .await
}
