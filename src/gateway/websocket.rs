//! OrA WebSocket Handler
//!
//! WebSocket handler for real-time communication with the frontend.
//! Streams agent execution events and handles bidirectional messaging.

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::gateway::tasks::spawn_prompt_task;
use crate::state::{AgentEvent, AppState};

/// WebSocket message from client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Ping message
    Ping { timestamp: String },

    /// Chat message to process
    Chat {
        message: String,
        session_id: Option<String>,
    },

    /// Cancel current task
    Cancel { task_id: String },

    /// Get pending approvals
    GetPending,

    /// Approval response
    ApprovalResponse {
        id: String,
        approved: bool,
        reason: Option<String>,
    },
}

/// WebSocket message to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsResponse {
    /// Connected confirmation
    Connected { timestamp: String },

    /// Pong response
    Pong { timestamp: String },

    /// Task started event
    TaskStarted { task_id: String, task: String },

    /// Plan created event
    PlanCreated {
        task_id: String,
        steps: Vec<PlanStepInfo>,
    },

    /// Step started event
    StepStarted {
        task_id: String,
        step_number: u32,
        tool: String,
        description: String,
    },

    /// Step completed event
    StepCompleted {
        task_id: String,
        step_number: u32,
        success: bool,
        output: String,
    },

    /// Task completed event
    TaskCompleted {
        task_id: String,
        success: bool,
        summary: String,
        steps_executed: u32,
    },

    /// Task cancelled event
    TaskCancelled { task_id: String, reason: String },

    /// Chat response acknowledgement
    ChatResponse {
        message: String,
        task_id: Option<String>,
        status: String,
    },

    /// Pending approvals list
    PendingApprovals { approvals: Vec<PendingApproval> },

    /// Approval request broadcast
    ApprovalRequest(ApprovalRequest),

    /// Approval result
    ApprovalResult {
        approval_id: String,
        approved: bool,
        reason: Option<String>,
    },

    /// Metrics update
    MetricsUpdate {
        cpu: f64,
        memory: f64,
        latency_ms: f64,
    },

    /// Error message
    Error {
        task_id: Option<String>,
        message: String,
    },
}

/// Plan step info for client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStepInfo {
    pub step_number: u32,
    pub tool: String,
    pub description: String,
}

/// Pending approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub agent: String,
    pub operation: String,
    pub description: String,
    pub authority_required: String,
    pub query: String,
    pub created_at: String,
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub agent: String,
    pub operation: String,
    pub description: String,
    pub authority_required: String,
    pub query: String,
}

/// Handle WebSocket upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State((_, state)): State<(crate::config::Config, AppState)>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<WsResponse>();

    let connected = WsResponse::Connected {
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let _ = outgoing_tx.send(connected);

    let writer = tokio::spawn(async move {
        while let Some(response) = outgoing_rx.recv().await {
            let json = match serde_json::to_string(&response) {
                Ok(json) => json,
                Err(_) => continue,
            };

            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let mut event_rx = state.subscribe();
    let event_tx = outgoing_tx.clone();
    let event_forwarder = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            if event_tx.send(event_to_response(&event)).is_err() {
                break;
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let response = match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => handle_message(ws_msg, &state).await,
                    Err(error) => WsResponse::Error {
                        task_id: None,
                        message: format!("Invalid WebSocket payload: {}", error),
                    },
                };

                if outgoing_tx.send(response).is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    event_forwarder.abort();
    drop(outgoing_tx);
    writer.abort();
}

/// Handle incoming message and return an immediate protocol response.
async fn handle_message(msg: WsMessage, state: &AppState) -> WsResponse {
    match msg {
        WsMessage::Ping { timestamp } => WsResponse::Pong { timestamp },
        WsMessage::Chat {
            message,
            session_id,
        } => {
            let task_id = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            std::mem::drop(spawn_prompt_task(state.clone(), task_id.clone(), message));
            WsResponse::ChatResponse {
                message: "Task accepted".to_string(),
                task_id: Some(task_id),
                status: "running".to_string(),
            }
        }
        WsMessage::Cancel { task_id } => {
            if state.cancel_task(&task_id, None) {
                WsResponse::TaskCancelled {
                    task_id,
                    reason: "Task cancelled by client".to_string(),
                }
            } else {
                WsResponse::Error {
                    task_id: Some(task_id),
                    message: "Task is not running or was not found".to_string(),
                }
            }
        }
        WsMessage::GetPending => WsResponse::PendingApprovals {
            approvals: state
                .pending_approvals()
                .into_iter()
                .map(|approval| PendingApproval {
                    id: approval.id,
                    agent: approval.agent,
                    operation: approval.operation,
                    description: approval.description,
                    authority_required: approval.authority_required,
                    query: approval.query,
                    created_at: approval.created_at,
                })
                .collect(),
        },
        WsMessage::ApprovalResponse {
            id,
            approved,
            reason,
        } => {
            let known = state
                .pending_approvals()
                .into_iter()
                .any(|approval| approval.id == id);
            if !known {
                return WsResponse::Error {
                    task_id: None,
                    message: format!("Approval {} not found", id),
                };
            }

            state.broadcast(AgentEvent::ApprovalResolved {
                approval_id: id.clone(),
                approved,
                approver: Some("websocket-client".to_string()),
                reason: reason.clone(),
            });

            WsResponse::ApprovalResult {
                approval_id: id,
                approved,
                reason,
            }
        }
    }
}

/// Convert AgentEvent to WsResponse.
fn event_to_response(event: &AgentEvent) -> WsResponse {
    match event {
        AgentEvent::TaskStarted { task_id, task } => WsResponse::TaskStarted {
            task_id: task_id.clone(),
            task: task.clone(),
        },
        AgentEvent::PlanCreated { task_id, steps } => WsResponse::PlanCreated {
            task_id: task_id.clone(),
            steps: steps
                .iter()
                .enumerate()
                .map(|(i, step)| PlanStepInfo {
                    step_number: i as u32 + 1,
                    tool: step.clone(),
                    description: step.clone(),
                })
                .collect(),
        },
        AgentEvent::StepStarted {
            task_id,
            step_number,
            tool,
            description,
        } => WsResponse::StepStarted {
            task_id: task_id.clone(),
            step_number: *step_number,
            tool: tool.clone(),
            description: description.clone(),
        },
        AgentEvent::StepCompleted {
            task_id,
            step_number,
            success,
            output,
        } => WsResponse::StepCompleted {
            task_id: task_id.clone(),
            step_number: *step_number,
            success: *success,
            output: output.clone(),
        },
        AgentEvent::TaskCompleted {
            task_id,
            success,
            summary,
        } => WsResponse::TaskCompleted {
            task_id: task_id.clone(),
            success: *success,
            summary: summary.clone(),
            steps_executed: 1,
        },
        AgentEvent::TaskCancelled { task_id, reason } => WsResponse::TaskCancelled {
            task_id: task_id.clone(),
            reason: reason.clone(),
        },
        AgentEvent::ApprovalRequired {
            approval_id,
            agent,
            operation,
            description,
            authority_required,
            query,
        } => WsResponse::ApprovalRequest(ApprovalRequest {
            id: approval_id.clone(),
            agent: agent.clone(),
            operation: operation.clone(),
            description: description.clone(),
            authority_required: authority_required.clone(),
            query: query.clone(),
        }),
        AgentEvent::ApprovalResolved {
            approval_id,
            approved,
            reason,
            ..
        } => WsResponse::ApprovalResult {
            approval_id: approval_id.clone(),
            approved: *approved,
            reason: reason.clone(),
        },
        AgentEvent::Error { task_id, message } => WsResponse::Error {
            task_id: task_id.clone(),
            message: message.clone(),
        },
    }
}
