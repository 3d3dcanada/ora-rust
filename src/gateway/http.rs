//! OrA HTTP Gateway
//!
//! REST API routes for OrA.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::gateway::tasks::spawn_prompt_task;
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
    pub description: String,
    pub authority_required: String,
    pub query: String,
    pub created_at: String,
    pub status: String,
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

async fn approvals_list(State((_, state)): State<(Config, AppState)>) -> Json<ApprovalsResponse> {
    Json(ApprovalsResponse {
        approvals: state
            .pending_approvals()
            .into_iter()
            .map(|approval| ApprovalInfo {
                id: approval.id,
                agent: approval.agent,
                operation: approval.operation,
                description: approval.description,
                authority_required: approval.authority_required,
                query: approval.query,
                created_at: approval.created_at,
                status: "pending".to_string(),
            })
            .collect(),
    })
}

async fn approval_approve(
    Path(id): Path<String>,
    State((_, state)): State<(Config, AppState)>,
    Json(req): Json<ApprovalActionRequest>,
) -> (StatusCode, Json<ApprovalActionResponse>) {
    let known = state
        .pending_approvals()
        .into_iter()
        .any(|approval| approval.id == id);
    if !known {
        return (
            StatusCode::NOT_FOUND,
            Json(ApprovalActionResponse {
                success: false,
                message: format!("Approval {} not found", id),
            }),
        );
    }

    state.broadcast(crate::state::AgentEvent::ApprovalResolved {
        approval_id: id.clone(),
        approved: true,
        approver: req.approver,
        reason: req.reason,
    });

    (
        StatusCode::OK,
        Json(ApprovalActionResponse {
            success: true,
            message: format!("Approval {} approved", id),
        }),
    )
}

async fn approval_reject(
    Path(id): Path<String>,
    State((_, state)): State<(Config, AppState)>,
    Json(req): Json<ApprovalActionRequest>,
) -> (StatusCode, Json<ApprovalActionResponse>) {
    let known = state
        .pending_approvals()
        .into_iter()
        .any(|approval| approval.id == id);
    if !known {
        return (
            StatusCode::NOT_FOUND,
            Json(ApprovalActionResponse {
                success: false,
                message: format!("Approval {} not found", id),
            }),
        );
    }

    state.broadcast(crate::state::AgentEvent::ApprovalResolved {
        approval_id: id.clone(),
        approved: false,
        approver: req.approver,
        reason: req.reason,
    });

    (
        StatusCode::OK,
        Json(ApprovalActionResponse {
            success: true,
            message: format!("Approval {} rejected", id),
        }),
    )
}

async fn kernel_metrics() -> Json<MetricsResponse> {
    Json(MetricsResponse {
        cpu_usage: current_cpu_usage(),
        memory_usage: current_memory_usage(),
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
    let command = if req.command.trim().is_empty() {
        req.message
    } else {
        req.command
    };

    let handle = spawn_prompt_task(state.clone(), task_id.clone(), command);

    match handle.await {
        Ok(result) => Json(ChatResponse {
            response: result.response,
            session_id: result.task_id,
            requires_approval: result.requires_approval,
        }),
        Err(error) if error.is_cancelled() => Json(ChatResponse {
            response: state
                .task(&task_id)
                .and_then(|task| task.summary)
                .unwrap_or_else(|| "Task cancelled by client".to_string()),
            session_id: task_id,
            requires_approval: false,
        }),
        Err(error) => Json(ChatResponse {
            response: format!("Task execution failed: {}", error),
            session_id: task_id,
            requires_approval: false,
        }),
    }
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

fn current_cpu_usage() -> f64 {
    #[cfg(target_os = "linux")]
    {
        let load = std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|contents| contents.split_whitespace().next()?.parse::<f64>().ok())
            .unwrap_or(0.0);
        let cpus = std::thread::available_parallelism()
            .map(|count| count.get() as f64)
            .unwrap_or(1.0);
        ((load / cpus) * 100.0).clamp(0.0, 100.0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        0.0
    }
}

fn current_memory_usage() -> f64 {
    #[cfg(target_os = "linux")]
    {
        let meminfo = match std::fs::read_to_string("/proc/meminfo") {
            Ok(contents) => contents,
            Err(_) => return 0.0,
        };

        let mut total_kb = None;
        let mut available_kb = None;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|value| value.parse::<f64>().ok());
            }
            if line.starts_with("MemAvailable:") {
                available_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|value| value.parse::<f64>().ok());
            }
        }

        match (total_kb, available_kb) {
            (Some(total), Some(available)) if total > 0.0 => {
                ((1.0 - (available / total)) * 100.0).clamp(0.0, 100.0)
            }
            _ => 0.0,
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        0.0
    }
}
