//! OrA Agent Module
//!
//! The main agent loop - ties together kernel, tools, memory, LLM, security, and audit.
//! Plan → Execute → Verify → Retry Loop

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::audit::AuditLogger;
use crate::error::{OraError, Result};
use crate::kernel::{memory::OraMemory, Kernel, ToolExecutor};
use crate::llm::LlmClient;
use crate::security::AstParser;

/// Maximum retry attempts per step
pub const MAX_RETRIES: u32 = 3;
/// Maximum steps per task
pub const MAX_STEPS: u32 = 20;
/// Maximum verification attempts
pub const MAX_VERIFY_ATTEMPTS: u32 = 2;

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Success,
    Failed,
    Verified,
    Retrying,
    Blocked,
}

/// A single execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number
    pub step_number: u32,
    /// Tool name to execute
    pub tool: String,
    /// Arguments for the tool
    pub args: serde_json::Value,
    /// Description of what this step does
    pub description: String,
    /// Expected outcome
    pub expected_outcome: String,
    /// Current status
    pub status: ExecutionStatus,
    /// Number of attempts
    pub attempts: u32,
    /// Verification attempts
    pub verify_attempts: u32,
    /// Result from tool execution
    pub result: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Verification error if failed
    pub verify_error: Option<String>,
}

impl ExecutionStep {
    /// Create a new step
    pub fn new(step_number: u32, tool: &str, args: serde_json::Value, description: &str) -> Self {
        Self {
            step_number,
            tool: tool.to_string(),
            args,
            description: description.to_string(),
            expected_outcome: String::new(),
            status: ExecutionStatus::Pending,
            attempts: 0,
            verify_attempts: 0,
            result: None,
            error: None,
            verify_error: None,
        }
    }

    /// Check if can retry
    pub fn can_retry(&self) -> bool {
        self.attempts < MAX_RETRIES
    }

    /// Check if can verify retry
    pub fn can_verify_retry(&self) -> bool {
        self.verify_attempts < MAX_VERIFY_ATTEMPTS
    }
}

/// A plan with multiple steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Plan ID
    pub id: String,
    /// Original task
    pub task: String,
    /// Steps to execute
    pub steps: Vec<ExecutionStep>,
    /// Current step index
    pub current_step: usize,
    /// Overall status
    pub status: ExecutionStatus,
    /// Goal/objective
    pub goal: String,
}

impl ExecutionPlan {
    /// Create a new plan
    pub fn new(task: &str, goal: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task: task.to_string(),
            steps: Vec::new(),
            current_step: 0,
            status: ExecutionStatus::Pending,
            goal: goal.to_string(),
        }
    }

    /// Add a step
    pub fn add_step(&mut self, tool: &str, args: serde_json::Value, description: &str) {
        let step = ExecutionStep::new(self.steps.len() as u32 + 1, tool, args, description);
        self.steps.push(step);
    }

    /// Get current step
    pub fn current_step(&self) -> Option<&ExecutionStep> {
        self.steps.get(self.current_step)
    }

    /// Get current step mut
    pub fn current_step_mut(&mut self) -> Option<&mut ExecutionStep> {
        self.steps.get_mut(self.current_step)
    }

    /// Move to next step
    pub fn next_step(&mut self) -> bool {
        if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
            true
        } else {
            false
        }
    }

    /// Check if all steps done
    pub fn is_complete(&self) -> bool {
        self.steps
            .iter()
            .all(|s| s.status == ExecutionStatus::Success || s.status == ExecutionStatus::Verified)
    }

    /// Check if any step failed
    pub fn has_failure(&self) -> bool {
        self.steps
            .iter()
            .any(|s| s.status == ExecutionStatus::Failed)
    }
}

/// Agent result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Plan that was executed
    pub plan: ExecutionPlan,
    /// Final output
    pub output: String,
    /// Total steps executed
    pub steps_executed: u32,
    /// Total retries
    pub total_retries: u32,
    /// Error message if failed
    pub error: Option<String>,
}

/// The main OrA Agent
pub struct Agent {
    /// Kernel (authority, constitution, validation)
    kernel: Arc<RwLock<Kernel>>,
    /// Tool executor
    tools: Arc<ToolExecutor>,
    /// Security gates
    gates: Arc<AstParser>,
    /// Audit logger
    audit: Arc<RwLock<AuditLogger>>,
    /// LLM client
    llm: Arc<LlmClient>,
    /// Memory
    memory: Arc<RwLock<OraMemory>>,
    /// Workspace root
    workspace: PathBuf,
    /// Auto-approve dangerous operations
    auto_approve: bool,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        kernel: Arc<RwLock<Kernel>>,
        tools: ToolExecutor,
        gates: Arc<AstParser>,
        audit: Arc<RwLock<AuditLogger>>,
        llm: Arc<LlmClient>,
        workspace: PathBuf,
    ) -> Self {
        let memory = Arc::new(RwLock::new(OraMemory::new_default()));

        Self {
            kernel,
            tools: Arc::new(tools),
            gates,
            audit,
            llm,
            memory,
            workspace,
            auto_approve: false,
        }
    }

    /// Set auto-approve mode
    pub fn with_auto_approve(mut self, auto: bool) -> Self {
        self.auto_approve = auto;
        self
    }

    /// Execute a task
    pub async fn execute(&mut self, task: &str) -> Result<AgentResult> {
        // 1. SECURITY: Run gates on input
        let gate_result = self.gates.parse_prompt(task);
        if !gate_result.passed {
            self.log_security_block(task, &gate_result).await?;
            return Err(OraError::SecurityBlocked {
                reason: gate_result
                    .reason
                    .unwrap_or_else(|| "Security gate failed".to_string()),
            });
        }

        // 2. Log start
        self.log_start(task).await?;

        // 3. Get context from memory
        let context = {
            let mem = self.memory.read().await;
            mem.get_context()
        };

        // 4. Create plan using LLM
        let plan = match self.create_plan(task, &context).await {
            Ok(p) => p,
            Err(e) => {
                self.log_error("plan_creation", &e.to_string()).await?;
                return Err(e);
            }
        };

        // 5. Log plan created
        self.log_plan(&plan).await?;

        // 6. Execute plan
        let result = self.execute_plan(plan, task).await;

        // 7. Add to memory
        {
            let mut mem = self.memory.write().await;
            mem.add_user(task);
            match &result {
                Ok(r) => {
                    if r.success {
                        mem.add_assistant(&format!(
                            "Task completed successfully. Steps: {}",
                            r.steps_executed
                        ));
                    } else {
                        mem.add_assistant(&format!(
                            "Task failed: {}",
                            r.error.as_deref().unwrap_or("Unknown")
                        ));
                    }
                }
                Err(e) => {
                    mem.add_assistant(&format!("Task error: {}", e));
                }
            }
        }

        result
    }

    /// Create execution plan from task
    async fn create_plan(&self, task: &str, context: &str) -> Result<ExecutionPlan> {
        let system_prompt = r#"You are OrA's planning module. Create a precise execution plan.

Output ONLY valid JSON in this exact format:
{
  "goal": "What success looks like",
  "steps": [
    {
      "tool": "tool_name",
      "args": {"key": "value"},
      "description": "What this step does",
      "expected": "Expected outcome"
    }
  ]
}

Available tools:
- read_file: {"path": "/path/to/file"}
- write_file: {"path": "/path/to/file", "content": "..."}
- execute_command: {"command": "shell command"}
- list_directory: {"path": "/path"}
- web_search: {"query": "search term"}
- analyze_code: {"code": "...", "language": "python"}

Create a plan that accomplishes the task. Be specific and practical."#;

        let user_msg = format!("Task: {}\n\nContext:\n{}", task, context);

        let response = self.llm.chat(system_prompt, &user_msg).await?;

        // Parse JSON from response
        let json_start = response.find('{');
        let json_end = response.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            if let Ok(plan_json) = serde_json::from_str::<serde_json::Value>(json_str) {
                let goal = plan_json
                    .get("goal")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Complete task")
                    .to_string();

                let mut plan = ExecutionPlan::new(task, &goal);

                if let Some(steps) = plan_json.get("steps").and_then(|v| v.as_array()) {
                    for step in steps {
                        let tool = step
                            .get("tool")
                            .and_then(|v| v.as_str())
                            .unwrap_or("execute_command");
                        let args = step.get("args").cloned().unwrap_or(serde_json::json!({}));
                        let description = step
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        plan.add_step(tool, args, description);
                    }
                }

                return Ok(plan);
            }
        }

        // Fallback: create simple plan with one command step
        let mut plan = ExecutionPlan::new(task, "Execute task");
        plan.add_step(
            "execute_command",
            serde_json::json!({ "command": task }),
            "Execute the task",
        );
        Ok(plan)
    }

    /// Execute the plan
    async fn execute_plan(&mut self, mut plan: ExecutionPlan, task: &str) -> Result<AgentResult> {
        let mut total_retries = 0u32;
        let mut steps_executed = 0u32;

        // Mark plan as running
        plan.status = ExecutionStatus::Running;

        while let Some(step) = plan.current_step_mut() {
            steps_executed += 1;

            // Mark step as running
            step.status = ExecutionStatus::Running;
            step.attempts += 1;

            self.log_step_start(step).await?;

            // Execute step
            let result = self.tools.execute(&step.tool, step.args.clone()).await;

            step.result = Some(result.output.clone());
            if !result.success {
                step.error = result.error.clone();
            }

            // Self-contained verification
            let verified = self.verify_step(step, &result).await;

            // Handle verification result
            if verified.success {
                step.status = ExecutionStatus::Verified;
                self.log_step_success(step).await?;

                // Move to next step
                plan.next_step();
            } else {
                step.verify_error = verified.error.clone();

                // Check if we should retry
                if step.can_retry() {
                    step.status = ExecutionStatus::Retrying;
                    total_retries += 1;

                    self.log_retry(step).await?;

                    // Try to revise the step
                    let error_msg = verified.error.as_deref().unwrap_or("Unknown error");
                    if let Ok(new_args) = self.revise_step(step, error_msg).await {
                        step.args = new_args;
                        // Retry same step
                        continue;
                    }
                } else {
                    step.status = ExecutionStatus::Failed;
                    self.log_step_failed(step).await?;

                    plan.status = ExecutionStatus::Failed;

                    return Ok(AgentResult {
                        success: false,
                        plan,
                        output: String::new(),
                        steps_executed,
                        total_retries,
                        error: verified.error,
                    });
                }
            }
        }

        // Plan complete
        if plan.has_failure() {
            plan.status = ExecutionStatus::Failed;
            Ok(AgentResult {
                success: false,
                plan,
                output: String::new(),
                steps_executed,
                total_retries,
                error: Some("Some steps failed".to_string()),
            })
        } else {
            plan.status = ExecutionStatus::Success;
            self.log_complete(task, steps_executed).await?;

            Ok(AgentResult {
                success: true,
                plan,
                output: "Task completed successfully".to_string(),
                steps_executed,
                total_retries,
                error: None,
            })
        }
    }

    /// Verify step result (self-contained)
    async fn verify_step(
        &self,
        step: &ExecutionStep,
        result: &super::tools::ToolResult,
    ) -> VerificationResult {
        // Basic verification based on tool type
        match step.tool.as_str() {
            "write_file" => {
                // Check if file was created
                if let Some(path) = step.args.get("path").and_then(|v| v.as_str()) {
                    let exists = std::path::Path::new(path).exists();
                    VerificationResult {
                        success: exists && result.success,
                        error: if !exists {
                            Some(format!("File not created: {}", path))
                        } else if !result.success {
                            result.error.clone()
                        } else {
                            None
                        },
                    }
                } else {
                    VerificationResult {
                        success: result.success,
                        error: result.error.clone(),
                    }
                }
            }
            "execute_command" => {
                // Check exit code via success flag
                VerificationResult {
                    success: result.success,
                    error: if result.success {
                        None
                    } else {
                        result.error.clone()
                    },
                }
            }
            "read_file" => {
                // Check if we got content
                VerificationResult {
                    success: result.success && !result.output.is_empty(),
                    error: if result.output.is_empty() {
                        Some("No content read".to_string())
                    } else {
                        result.error.clone()
                    },
                }
            }
            _ => {
                // Default: trust the result
                VerificationResult {
                    success: result.success,
                    error: result.error.clone(),
                }
            }
        }
    }

    /// Revise step after failure
    async fn revise_step(&self, step: &ExecutionStep, error: &str) -> Result<serde_json::Value> {
        let prompt = format!(
            r#"A step failed. Revise the arguments to fix it.

Step: {} - {}
Error: {}
Current args: {}

Respond with ONLY valid JSON containing revised arguments."#,
            step.tool, step.description, error, step.args
        );

        let response = self
            .llm
            .chat("You are a helpful assistant.", &prompt)
            .await?;

        // Try to parse JSON
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                if let Ok(args) = serde_json::from_str(json_str) {
                    return Ok(args);
                }
            }
        }

        Err(OraError::AgentError {
            message: "Failed to parse revised args".to_string(),
        })
    }

    // ===== AUDIT LOGGING =====

    async fn log_start(&self, _task: &str) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry("OPERATION", "agent_start", "agent", "A0", "started")
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_plan(&self, _plan: &ExecutionPlan) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry("OPERATION", "plan_created", "agent", "A0", "success")
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_step_start(&self, step: &ExecutionStep) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry(
                "OPERATION",
                &format!("step_{}_start", step.step_number),
                &step.tool,
                "A2",
                "running",
            )
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_step_success(&self, step: &ExecutionStep) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry(
                "OPERATION",
                &format!("step_{}_success", step.step_number),
                &step.tool,
                "A2",
                "verified",
            )
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_step_failed(&self, step: &ExecutionStep) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry(
                "ERROR",
                &format!("step_{}_failed", step.step_number),
                &step.tool,
                "A2",
                "failed",
            )
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_retry(&self, step: &ExecutionStep) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry(
                "WARNING",
                &format!("step_{}_retry", step.step_number),
                &step.tool,
                "A2",
                "retrying",
            )
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_complete(&self, _task: &str, _steps: u32) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry("OPERATION", "agent_complete", "agent", "A0", "success")
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_error(&self, action: &str, _error: &str) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry("ERROR", action, "agent", "A0", "error")
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }

    async fn log_security_block(
        &self,
        _input: &str,
        _result: &crate::security::gates::GateResult,
    ) -> Result<()> {
        let mut audit = self.audit.write().await;
        audit
            .log_entry("SECURITY", "input_blocked", "gates", "A0", "blocked")
            .map_err(|e| OraError::AuditWriteFailed {
                reason: e.to_string(),
            })
    }
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub success: bool,
    pub error: Option<String>,
}
