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

    /// Chat response (simplified)
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

    // Send connected message
    let connected = WsResponse::Connected {
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let _ = sender
        .send(Message::Text(
            serde_json::to_string(&connected).unwrap_or_default(),
        ))
        .await;

    // Subscribe to agent events
    let mut event_rx = state.subscribe();

    // Clone state for the event loop
    let state_clone = state.clone();

    // Spawn task to forward events to client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let response = event_to_response(&event);
            let json = serde_json::to_string(&response).unwrap_or_default();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    handle_message(ws_msg, &state_clone).await;
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    // Clean up
    send_task.abort();
}

/// Handle incoming message
async fn handle_message(msg: WsMessage, state: &AppState) {
    match msg {
        WsMessage::Ping { timestamp } => {
            // Pong is handled by the event forwarder
            let pong = WsResponse::Pong { timestamp };
            state.broadcast(AgentEvent::Error {
                task_id: None,
                message: serde_json::to_string(&pong).unwrap_or_default(),
            });
        }

        WsMessage::Chat {
            message,
            session_id,
        } => {
            // The actual processing is done via HTTP endpoint
            // This just acknowledges receipt
            let _ = state.event_tx.send(AgentEvent::TaskStarted {
                task_id: session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                task: message,
            });
        }

        WsMessage::Cancel { task_id } => {
            // TODO: Implement task cancellation
            let _ = state.event_tx.send(AgentEvent::Error {
                task_id: Some(task_id),
                message: "Task cancelled".to_string(),
            });
        }

        WsMessage::GetPending => {
            // Return empty pending list (would query approval system)
        }

        WsMessage::ApprovalResponse {
            id,
            approved,
            reason,
        } => {
            // Handle approval response
            let _ = state.event_tx.send(AgentEvent::ApprovalRequired {
                approval_id: id,
                operation: if approved { "approved" } else { "rejected" }.to_string(),
                description: reason.unwrap_or_default(),
            });
        }
    }
}

/// Convert AgentEvent to WsResponse
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
                .map(|(i, s)| PlanStepInfo {
                    step_number: i as u32 + 1,
                    tool: s.clone(), // Simplified
                    description: s.clone(),
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
            steps_executed: 0, // Would be filled from actual result
        },

        AgentEvent::ApprovalRequired {
            approval_id,
            operation,
            description,
        } => WsResponse::ApprovalRequest(ApprovalRequest {
            id: approval_id.clone(),
            agent: "OrA".to_string(),
            operation: operation.clone(),
            description: description.clone(),
            authority_required: "A3".to_string(),
            query: description.clone(),
        }),

        AgentEvent::Error { task_id, message } => WsResponse::Error {
            task_id: task_id.clone(),
            message: message.clone(),
        },
    }
}
