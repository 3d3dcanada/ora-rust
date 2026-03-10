//! OrA Application State

use crate::audit::logger::AuditLogger;
use crate::config::Config;
use crate::kernel::Kernel;
use crate::llm::LlmClient;
use crate::security::gates::AstParser;
use crate::security::vault::Vault;
use chrono::Utc;
use parking_lot::RwLock as ParkingRwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::AbortHandle;

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
    /// Task cancelled
    TaskCancelled { task_id: String, reason: String },
    /// Approval required
    ApprovalRequired {
        approval_id: String,
        agent: String,
        operation: String,
        description: String,
        authority_required: String,
        query: String,
    },
    /// Approval resolved
    ApprovalResolved {
        approval_id: String,
        approved: bool,
        approver: Option<String>,
        reason: Option<String>,
    },
    /// Error occurred
    Error {
        task_id: Option<String>,
        message: String,
    },
}

/// Runtime task lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Serializable task snapshot for API consumers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub command: String,
    pub status: TaskStatus,
    pub started_at: String,
    pub updated_at: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone)]
struct ManagedTask {
    info: TaskInfo,
    abort_handle: Option<AbortHandle>,
}

/// Approval lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

/// Approval record shared by HTTP and WebSocket APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: String,
    pub agent: String,
    pub operation: String,
    pub description: String,
    pub authority_required: String,
    pub query: String,
    pub created_at: String,
    pub status: ApprovalStatus,
    pub approver: Option<String>,
    pub reason: Option<String>,
    pub resolved_at: Option<String>,
}

impl ApprovalRecord {
    fn pending(
        approval_id: String,
        agent: String,
        operation: String,
        description: String,
        authority_required: String,
        query: String,
    ) -> Self {
        Self {
            id: approval_id,
            agent,
            operation,
            description,
            authority_required,
            query,
            created_at: Utc::now().to_rfc3339(),
            status: ApprovalStatus::Pending,
            approver: None,
            reason: None,
            resolved_at: None,
        }
    }
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
    tasks: Arc<ParkingRwLock<HashMap<String, ManagedTask>>>,
    approvals: Arc<ParkingRwLock<HashMap<String, ApprovalRecord>>>,
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
            tasks: Arc::new(ParkingRwLock::new(HashMap::new())),
            approvals: Arc::new(ParkingRwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to agent events.
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// Register a long-running task so it can be observed or cancelled.
    pub fn register_task(
        &self,
        task_id: String,
        command: String,
        abort_handle: Option<AbortHandle>,
    ) {
        let now = Utc::now().to_rfc3339();
        let info = TaskInfo {
            id: task_id.clone(),
            command,
            status: TaskStatus::Running,
            started_at: now.clone(),
            updated_at: now,
            summary: None,
        };

        self.tasks
            .write()
            .insert(task_id, ManagedTask { info, abort_handle });
    }

    /// Attach an abort handle to an already-registered task.
    pub fn attach_abort_handle(&self, task_id: &str, abort_handle: AbortHandle) {
        let should_abort = {
            let mut tasks = self.tasks.write();
            match tasks.get_mut(task_id) {
                Some(task) if task.info.status == TaskStatus::Running => {
                    task.abort_handle = Some(abort_handle.clone());
                    false
                }
                Some(task) if task.info.status == TaskStatus::Cancelled => true,
                _ => true,
            }
        };

        if should_abort {
            abort_handle.abort();
        }
    }

    /// Mark a task as finished.
    pub fn finish_task(&self, task_id: &str, status: TaskStatus, summary: Option<String>) {
        if let Some(task) = self.tasks.write().get_mut(task_id) {
            if task.info.status == TaskStatus::Cancelled {
                return;
            }

            task.info.status = status;
            task.info.summary = summary;
            task.info.updated_at = Utc::now().to_rfc3339();
            task.abort_handle = None;
        }
    }

    /// Cancel a running task.
    pub fn cancel_task(&self, task_id: &str, reason: Option<String>) -> bool {
        let reason = reason.unwrap_or_else(|| "Task cancelled by client".to_string());

        let cancelled = {
            let mut tasks = self.tasks.write();
            match tasks.get_mut(task_id) {
                Some(task) if task.info.status == TaskStatus::Running => {
                    if let Some(abort_handle) = task.abort_handle.take() {
                        abort_handle.abort();
                    }
                    task.info.status = TaskStatus::Cancelled;
                    task.info.summary = Some(reason.clone());
                    task.info.updated_at = Utc::now().to_rfc3339();
                    true
                }
                _ => false,
            }
        };

        if cancelled {
            self.broadcast(AgentEvent::TaskCancelled {
                task_id: task_id.to_string(),
                reason,
            });
        }

        cancelled
    }

    /// Fetch a task snapshot if it exists.
    pub fn task(&self, task_id: &str) -> Option<TaskInfo> {
        self.tasks.read().get(task_id).map(|task| task.info.clone())
    }

    /// Record a pending approval.
    pub fn record_approval(&self, approval: ApprovalRecord) {
        self.approvals.write().insert(approval.id.clone(), approval);
    }

    /// Return all pending approvals in creation order.
    pub fn pending_approvals(&self) -> Vec<ApprovalRecord> {
        let mut approvals = self
            .approvals
            .read()
            .values()
            .filter(|approval| approval.status == ApprovalStatus::Pending)
            .cloned()
            .collect::<Vec<_>>();

        approvals.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        approvals
    }

    /// Resolve an approval and return the updated record.
    pub fn resolve_approval(
        &self,
        approval_id: &str,
        approved: bool,
        approver: Option<String>,
        reason: Option<String>,
    ) -> Option<ApprovalRecord> {
        let mut approvals = self.approvals.write();
        let approval = approvals.get_mut(approval_id)?;
        approval.status = if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };
        approval.approver = approver;
        approval.reason = reason;
        approval.resolved_at = Some(Utc::now().to_rfc3339());
        Some(approval.clone())
    }

    /// Broadcast an event and keep approval state in sync.
    pub fn broadcast(&self, event: AgentEvent) {
        match &event {
            AgentEvent::ApprovalRequired {
                approval_id,
                agent,
                operation,
                description,
                authority_required,
                query,
            } => {
                self.record_approval(ApprovalRecord::pending(
                    approval_id.clone(),
                    agent.clone(),
                    operation.clone(),
                    description.clone(),
                    authority_required.clone(),
                    query.clone(),
                ));
            }
            AgentEvent::ApprovalResolved {
                approval_id,
                approved,
                approver,
                reason,
            } => {
                let _ =
                    self.resolve_approval(approval_id, *approved, approver.clone(), reason.clone());
            }
            _ => {}
        }

        // Ignore send errors (no subscribers)
        let _ = self.event_tx.send(event);
    }
}
