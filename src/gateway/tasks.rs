//! Shared task execution utilities for HTTP and WebSocket gateways.

use crate::error::OraError;
use crate::state::{AgentEvent, AppState, TaskStatus};

/// Result returned to gateway callers after prompt execution finishes.
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    pub task_id: String,
    pub response: String,
    pub success: bool,
    pub requires_approval: bool,
}

/// Spawn a prompt-processing task and register it for cancellation.
pub fn spawn_prompt_task(
    state: AppState,
    task_id: String,
    command: String,
) -> tokio::task::JoinHandle<TaskExecutionResult> {
    state.register_task(task_id.clone(), command.clone(), None);

    state.broadcast(AgentEvent::TaskStarted {
        task_id: task_id.clone(),
        task: command.clone(),
    });

    let state_for_task = state.clone();
    let task_id_for_task = task_id.clone();
    let command_for_task = command.clone();

    let handle = tokio::spawn(async move {
        let gate_result = state_for_task
            .security_gates
            .parse_prompt(&command_for_task);
        if !gate_result.passed {
            let reason = gate_result
                .reason
                .unwrap_or_else(|| "Security gate failed".to_string());
            let message = format!("Security blocked: {}", reason);
            state_for_task.finish_task(
                &task_id_for_task,
                TaskStatus::Failed,
                Some(message.clone()),
            );
            state_for_task.broadcast(AgentEvent::Error {
                task_id: Some(task_id_for_task.clone()),
                message: message.clone(),
            });

            return TaskExecutionResult {
                task_id: task_id_for_task,
                response: message,
                success: false,
                requires_approval: false,
            };
        }

        match generate_chat_response(&state_for_task, &command_for_task).await {
            Ok(output) => {
                state_for_task.finish_task(
                    &task_id_for_task,
                    TaskStatus::Completed,
                    Some(output.clone()),
                );
                state_for_task.broadcast(AgentEvent::TaskCompleted {
                    task_id: task_id_for_task.clone(),
                    success: true,
                    summary: output.clone(),
                });

                TaskExecutionResult {
                    task_id: task_id_for_task,
                    response: output,
                    success: true,
                    requires_approval: false,
                }
            }
            Err(error) => {
                let message = map_chat_error(error);
                state_for_task.finish_task(
                    &task_id_for_task,
                    TaskStatus::Failed,
                    Some(message.clone()),
                );
                state_for_task.broadcast(AgentEvent::Error {
                    task_id: Some(task_id_for_task.clone()),
                    message: message.clone(),
                });

                TaskExecutionResult {
                    task_id: task_id_for_task,
                    response: message,
                    success: false,
                    requires_approval: false,
                }
            }
        }
    });

    state.attach_abort_handle(&task_id, handle.abort_handle());
    handle
}

fn ora_system_prompt() -> &'static str {
    r#"You are OrA, an AI assistant with a security-first mindset.
You help users with tasks while being aware of security implications.
Be concise and helpful. If a request seems dangerous, warn the user."#
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
