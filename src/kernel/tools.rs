//! OrA Tools Module
//!
//! Governed tool execution for filesystem, shell, memory, and higher-level runtime actions.

use crate::runtime::{
    create_browser_task, generate_verified_answer, grounded_summarize, BrowserTaskRequest,
};
use crate::state::AppState;
use crate::kernel::ApprovalRequest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Tool execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    #[serde(default)]
    pub requires_approval: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Tool executor.
pub struct ToolExecutor {
    state: AppState,
}

impl ToolExecutor {
    /// Create a new tool executor.
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Execute a tool.
    pub async fn execute(&self, tool_name: &str, arguments: serde_json::Value) -> ToolResult {
        match tool_name {
            "verified_answer" => self.verified_answer(arguments).await,
            "grounded_summarize" => self.grounded_summarize(arguments).await,
            "memory_search" => self.memory_search(arguments).await,
            "safe_browser_task" => self.safe_browser_task(arguments).await,
            "approval_queue" => self.approval_queue(arguments).await,
            "evidence_bundle" => self.evidence_bundle(arguments).await,
            "create_mission" => self.create_mission(arguments).await,
            "list_missions" => self.list_missions(arguments).await,
            "read_file" => self.read_file(arguments).await,
            "write_file" => self.write_file(arguments).await,
            "list_directory" => self.list_directory(arguments).await,
            "execute_command" => self.execute_command(arguments).await,
            "web_search" => self.web_search(arguments).await,
            "analyze_code" => self.analyze_code(arguments).await,
            "get_system_info" => self.get_system_info(arguments).await,
            _ => ToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                output: String::new(),
                error: Some(format!("Unknown tool: {tool_name}")),
                requires_approval: false,
                approval_id: None,
                metadata: None,
            },
        }
    }

    async fn verified_answer(&self, args: serde_json::Value) -> ToolResult {
        let Some(query) = args.get("query").and_then(|value| value.as_str()) else {
            return missing_argument("verified_answer", "query");
        };

        match generate_verified_answer(&self.state, query).await {
            Ok(result) => ToolResult {
                tool_name: "verified_answer".to_string(),
                success: true,
                output: result.response,
                error: None,
                requires_approval: false,
                approval_id: None,
                metadata: Some(json!({
                    "route_decision": result.route_decision,
                    "evidence_bundle_id": result.evidence_bundle_id,
                    "artifact_path": result.artifact_path,
                    "evidence_count": result.evidence.len(),
                })),
            },
            Err(error) => error_result("verified_answer", error),
        }
    }

    async fn grounded_summarize(&self, args: serde_json::Value) -> ToolResult {
        let Some(text) = args.get("text").and_then(|value| value.as_str()) else {
            return missing_argument("grounded_summarize", "text");
        };
        let title = args
            .get("title")
            .and_then(|value| value.as_str())
            .unwrap_or("Provided content");

        match grounded_summarize(&self.state, title, text).await {
            Ok(result) => ToolResult {
                tool_name: "grounded_summarize".to_string(),
                success: true,
                output: result.response,
                error: None,
                requires_approval: false,
                approval_id: None,
                metadata: Some(json!({
                    "evidence_bundle_id": result.evidence_bundle_id,
                    "route_decision": result.route_decision,
                })),
            },
            Err(error) => error_result("grounded_summarize", error),
        }
    }

    async fn memory_search(&self, args: serde_json::Value) -> ToolResult {
        let Some(query) = args.get("query").and_then(|value| value.as_str()) else {
            return missing_argument("memory_search", "query");
        };
        let limit = args
            .get("limit")
            .and_then(|value| value.as_u64())
            .map(|value| value as usize)
            .unwrap_or(5);

        let records = self.state.search_memory(query, limit);
        ToolResult {
            tool_name: "memory_search".to_string(),
            success: true,
            output: serde_json::to_string_pretty(&records).unwrap_or_else(|_| "[]".to_string()),
            error: None,
            requires_approval: false,
            approval_id: None,
            metadata: Some(json!({ "count": records.len() })),
        }
    }

    async fn safe_browser_task(&self, args: serde_json::Value) -> ToolResult {
        let Some(task) = args.get("task").and_then(|value| value.as_str()) else {
            return missing_argument("safe_browser_task", "task");
        };
        let url = args
            .get("url")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());
        let allowed_domains = args
            .get("allowed_domains")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(|value| value.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        match create_browser_task(
            &self.state,
            BrowserTaskRequest {
                task: task.to_string(),
                url,
                allowed_domains,
            },
        ) {
            Ok(result) => ToolResult {
                tool_name: "safe_browser_task".to_string(),
                success: true,
                output: result.message,
                error: None,
                requires_approval: result.approval_id.is_some(),
                approval_id: result.approval_id,
                metadata: Some(json!({ "task": result.task })),
            },
            Err(error) => error_result("safe_browser_task", error),
        }
    }

    async fn approval_queue(&self, _args: serde_json::Value) -> ToolResult {
        let approvals = self.state.pending_approvals();
        ToolResult {
            tool_name: "approval_queue".to_string(),
            success: true,
            output: serde_json::to_string_pretty(&approvals).unwrap_or_else(|_| "[]".to_string()),
            error: None,
            requires_approval: false,
            approval_id: None,
            metadata: Some(json!({ "count": approvals.len() })),
        }
    }

    async fn evidence_bundle(&self, args: serde_json::Value) -> ToolResult {
        let Some(bundle_id) = args.get("bundle_id").and_then(|value| value.as_str()) else {
            return missing_argument("evidence_bundle", "bundle_id");
        };

        let evidence = self.state.evidence_bundle(bundle_id);
        ToolResult {
            tool_name: "evidence_bundle".to_string(),
            success: true,
            output: serde_json::to_string_pretty(&evidence).unwrap_or_else(|_| "[]".to_string()),
            error: None,
            requires_approval: false,
            approval_id: None,
            metadata: Some(json!({ "count": evidence.len() })),
        }
    }

    async fn create_mission(&self, args: serde_json::Value) -> ToolResult {
        let Some(name) = args.get("name").and_then(|value| value.as_str()) else {
            return missing_argument("create_mission", "name");
        };
        let Some(query) = args.get("query").and_then(|value| value.as_str()) else {
            return missing_argument("create_mission", "query");
        };
        let freshness_policy = args
            .get("freshness_policy")
            .and_then(|value| value.as_str())
            .unwrap_or("recent");
        let sources = args
            .get("sources")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(|value| value.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let extraction_rules = args
            .get("extraction_rules")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(|value| value.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        match crate::runtime::create_mission(
            &self.state,
            name,
            query,
            sources,
            extraction_rules,
            freshness_policy,
        ) {
            Ok(mission) => ToolResult {
                tool_name: "create_mission".to_string(),
                success: true,
                output: serde_json::to_string_pretty(&mission)
                    .unwrap_or_else(|_| "{}".to_string()),
                error: None,
                requires_approval: false,
                approval_id: None,
                metadata: Some(json!({ "mission_id": mission.id })),
            },
            Err(error) => error_result("create_mission", error),
        }
    }

    async fn list_missions(&self, args: serde_json::Value) -> ToolResult {
        let limit = args
            .get("limit")
            .and_then(|value| value.as_u64())
            .map(|value| value as usize)
            .unwrap_or(10);
        let missions = self.state.list_missions(limit);
        ToolResult {
            tool_name: "list_missions".to_string(),
            success: true,
            output: serde_json::to_string_pretty(&missions).unwrap_or_else(|_| "[]".to_string()),
            error: None,
            requires_approval: false,
            approval_id: None,
            metadata: Some(json!({ "count": missions.len() })),
        }
    }

    async fn read_file(&self, args: serde_json::Value) -> ToolResult {
        let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
            return missing_argument("read_file", "path");
        };

        match self.resolve_existing_workspace_path(path) {
            Ok(path) => match std::fs::read_to_string(&path) {
                Ok(content) => ToolResult {
                    tool_name: "read_file".to_string(),
                    success: true,
                    output: content,
                    error: None,
                    requires_approval: false,
                    approval_id: None,
                    metadata: Some(json!({ "path": path })),
                },
                Err(error) => ToolResult {
                    tool_name: "read_file".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                    requires_approval: false,
                    approval_id: None,
                    metadata: None,
                },
            },
            Err(error) => ToolResult {
                tool_name: "read_file".to_string(),
                success: false,
                output: String::new(),
                error: Some(error),
                requires_approval: false,
                approval_id: None,
                metadata: None,
            },
        }
    }

    async fn write_file(&self, args: serde_json::Value) -> ToolResult {
        let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
            return missing_argument("write_file", "path");
        };
        let Some(content) = args.get("content").and_then(|value| value.as_str()) else {
            return missing_argument("write_file", "content");
        };

        match self.resolve_workspace_path(path) {
            Ok(path) => {
                if let Some(parent) = path.parent() {
                    if let Err(error) = std::fs::create_dir_all(parent) {
                        return ToolResult {
                            tool_name: "write_file".to_string(),
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                            requires_approval: false,
                            approval_id: None,
                            metadata: None,
                        };
                    }
                }

                match std::fs::write(&path, content) {
                    Ok(_) => ToolResult {
                        tool_name: "write_file".to_string(),
                        success: true,
                        output: format!("Written {} bytes", content.len()),
                        error: None,
                        requires_approval: false,
                        approval_id: None,
                        metadata: Some(json!({ "path": path })),
                    },
                    Err(error) => ToolResult {
                        tool_name: "write_file".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some(error.to_string()),
                        requires_approval: false,
                        approval_id: None,
                        metadata: None,
                    },
                }
            }
            Err(error) => ToolResult {
                tool_name: "write_file".to_string(),
                success: false,
                output: String::new(),
                error: Some(error),
                requires_approval: false,
                approval_id: None,
                metadata: None,
            },
        }
    }

    async fn list_directory(&self, args: serde_json::Value) -> ToolResult {
        let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
            return missing_argument("list_directory", "path");
        };

        match self.resolve_existing_workspace_path(path) {
            Ok(path) => match std::fs::read_dir(&path) {
                Ok(entries) => {
                    let mut output = String::new();
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        let kind = if entry_path.is_dir() { "dir" } else { "file" };
                        let _ = writeln!(output, "{}: {}", kind, entry_path.display());
                    }
                    ToolResult {
                        tool_name: "list_directory".to_string(),
                        success: true,
                        output,
                        error: None,
                        requires_approval: false,
                        approval_id: None,
                        metadata: Some(json!({ "path": path })),
                    }
                }
                Err(error) => ToolResult {
                    tool_name: "list_directory".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                    requires_approval: false,
                    approval_id: None,
                    metadata: None,
                },
            },
            Err(error) => ToolResult {
                tool_name: "list_directory".to_string(),
                success: false,
                output: String::new(),
                error: Some(error),
                requires_approval: false,
                approval_id: None,
                metadata: None,
            },
        }
    }

    async fn execute_command(&self, args: serde_json::Value) -> ToolResult {
        let Some(command) = args.get("command").and_then(|value| value.as_str()) else {
            return missing_argument("execute_command", "command");
        };

        if contains_shell_metacharacters(command) {
            return ToolResult {
                tool_name: "execute_command".to_string(),
                success: false,
                output: String::new(),
                error: Some(
                    "Shell metacharacters are not allowed. Pass a single explicit command."
                        .to_string(),
                ),
                requires_approval: false,
                approval_id: None,
                metadata: None,
            };
        }

        let parsed = match shell_words::split(command) {
            Ok(parts) if !parts.is_empty() => parts,
            Ok(_) => {
                return ToolResult {
                    tool_name: "execute_command".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some("Empty command".to_string()),
                    requires_approval: false,
                    approval_id: None,
                    metadata: None,
                }
            }
            Err(error) => {
                return ToolResult {
                    tool_name: "execute_command".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to parse command: {error}")),
                    requires_approval: false,
                    approval_id: None,
                    metadata: None,
                }
            }
        };

        let program = &parsed[0];
        let args = &parsed[1..];
        let policy = match classify_command(program, args) {
            Ok(policy) => policy,
            Err(error) => {
                return ToolResult {
                    tool_name: "execute_command".to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(error),
                    requires_approval: false,
                    approval_id: None,
                    metadata: None,
                }
            }
        };

        if policy.requires_approval {
            let approval = ApprovalRequest {
                id: crate::kernel::make_id(),
                operation: "execute_command".to_string(),
                action_class: policy.action_class.clone(),
                risk_class: policy.risk_class.clone(),
                authority_required: "A3".to_string(),
                description: format!(
                    "Approve command execution: {} {}",
                    program,
                    args.join(" ")
                )
                .trim()
                .to_string(),
                request_payload: json!({
                    "command": command,
                    "program": program,
                    "args": args,
                    "action_class": policy.action_class,
                }),
                status: crate::kernel::ApprovalState::Pending,
                created_at: chrono::Utc::now().to_rfc3339(),
                resolved_at: None,
                approver: None,
                resolution_reason: None,
            };
            let approval_id = approval.id.clone();
            self.state.queue_approval_request(approval);
            return ToolResult {
                tool_name: "execute_command".to_string(),
                success: true,
                output: "Command requires approval before execution.".to_string(),
                error: None,
                requires_approval: true,
                approval_id: Some(approval_id),
                metadata: Some(json!({
                    "command": command,
                    "action_class": policy.action_class,
                })),
            };
        }

        let output = Command::new(program)
            .args(args)
            .current_dir(&self.state.config.workspace_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                ToolResult {
                    tool_name: "execute_command".to_string(),
                    success: output.status.success(),
                    output: stdout,
                    error: if stderr.trim().is_empty() {
                        None
                    } else {
                        Some(stderr)
                    },
                    requires_approval: false,
                    approval_id: None,
                    metadata: Some(json!({
                        "program": program,
                        "args": args,
                        "action_class": policy.action_class,
                    })),
                }
            }
            Err(error) => ToolResult {
                tool_name: "execute_command".to_string(),
                success: false,
                output: String::new(),
                error: Some(error.to_string()),
                requires_approval: false,
                approval_id: None,
                metadata: None,
            },
        }
    }

    async fn web_search(&self, args: serde_json::Value) -> ToolResult {
        let Some(query) = args.get("query").and_then(|value| value.as_str()) else {
            return missing_argument("web_search", "query");
        };

        match generate_verified_answer(&self.state, query).await {
            Ok(result) => ToolResult {
                tool_name: "web_search".to_string(),
                success: true,
                output: result.response,
                error: None,
                requires_approval: false,
                approval_id: None,
                metadata: Some(json!({
                    "route_decision": result.route_decision,
                    "evidence_bundle_id": result.evidence_bundle_id,
                })),
            },
            Err(error) => error_result("web_search", error),
        }
    }

    async fn analyze_code(&self, args: serde_json::Value) -> ToolResult {
        let Some(code) = args.get("code").and_then(|value| value.as_str()) else {
            return missing_argument("analyze_code", "code");
        };

        let mut issues = Vec::new();
        if code.contains("eval(") {
            issues.push("Warning: use of eval() is dangerous");
        }
        if code.contains("exec(") {
            issues.push("Warning: use of exec() requires caution");
        }
        if code.contains("password") && code.contains('=') {
            issues.push("Warning: potential hardcoded password");
        }
        if code.contains("api_key") && code.contains('=') {
            issues.push("Warning: potential hardcoded API key");
        }

        ToolResult {
            tool_name: "analyze_code".to_string(),
            success: true,
            output: if issues.is_empty() {
                "No obvious issues found".to_string()
            } else {
                issues.join("\n")
            },
            error: None,
            requires_approval: false,
            approval_id: None,
            metadata: Some(json!({ "issues_found": issues.len() })),
        }
    }

    async fn get_system_info(&self, args: serde_json::Value) -> ToolResult {
        let info_type = args
            .get("info_type")
            .and_then(|value| value.as_str())
            .unwrap_or("all");

        let output = match info_type {
            "cpu" => format!("CPU: {} cores", num_cpus()),
            "memory" => format!("Memory: {} MB available", memory_info()),
            "disk" => format!("Disk: {} available", disk_info()),
            "all" => format!(
                "System Info:\n- CPU: {} cores\n- Memory: {} MB\n- Disk: {}",
                num_cpus(),
                memory_info(),
                disk_info()
            ),
            _ => "Unknown info type".to_string(),
        };

        ToolResult {
            tool_name: "get_system_info".to_string(),
            success: true,
            output,
            error: None,
            requires_approval: false,
            approval_id: None,
            metadata: None,
        }
    }

    fn resolve_workspace_path(&self, path: &str) -> std::result::Result<PathBuf, String> {
        let candidate = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.state.config.workspace_root.join(path)
        };

        if !candidate.starts_with(&self.state.config.workspace_root) {
            return Err("Access denied: path outside workspace".to_string());
        }

        Ok(candidate)
    }

    fn resolve_existing_workspace_path(&self, path: &str) -> std::result::Result<PathBuf, String> {
        let path = self.resolve_workspace_path(path)?;
        let canonical = std::fs::canonicalize(&path).map_err(|error| error.to_string())?;
        if !canonical.starts_with(&self.state.config.workspace_root) {
            return Err("Access denied: path outside workspace".to_string());
        }
        Ok(canonical)
    }
}

#[derive(Debug, Clone)]
struct CommandPolicy {
    action_class: String,
    risk_class: crate::kernel::RiskClass,
    requires_approval: bool,
}

fn classify_command(program: &str, args: &[String]) -> std::result::Result<CommandPolicy, String> {
    let read_only = [
        "ls", "pwd", "cat", "head", "tail", "rg", "fd", "find", "stat", "du",
    ];
    if read_only.contains(&program) {
        return Ok(CommandPolicy {
            action_class: "read_only".to_string(),
            risk_class: crate::kernel::RiskClass::Low,
            requires_approval: false,
        });
    }

    match program {
        "git" => classify_git(args),
        "cargo" => classify_cargo(args),
        "npm" | "pnpm" | "yarn" => classify_node_pkg(program, args),
        "mkdir" | "touch" | "cp" | "mv" => Ok(CommandPolicy {
            action_class: "workspace_mutation".to_string(),
            risk_class: crate::kernel::RiskClass::Medium,
            requires_approval: true,
        }),
        "rm" => Ok(CommandPolicy {
            action_class: "destructive".to_string(),
            risk_class: crate::kernel::RiskClass::High,
            requires_approval: true,
        }),
        "python" | "python3" | "node" | "bash" | "sh" | "curl" | "wget" | "ssh" | "scp"
        | "sudo" | "chmod" | "chown" | "docker" => Err(format!(
            "Program '{}' is not allowed in governed command mode",
            program
        )),
        _ => Err(format!(
            "Program '{}' is not on the ORA allowlist. Use a higher-level tool instead.",
            program
        )),
    }
}

fn classify_git(args: &[String]) -> std::result::Result<CommandPolicy, String> {
    let subcommand = args.first().map(String::as_str).unwrap_or("status");
    match subcommand {
        "status" | "diff" | "log" | "show" | "rev-parse" | "branch" | "remote" => {
            Ok(CommandPolicy {
                action_class: "read_only".to_string(),
                risk_class: crate::kernel::RiskClass::Low,
                requires_approval: false,
            })
        }
        "add" | "commit" | "merge" | "rebase" | "push" | "pull" | "switch" | "checkout"
        | "restore" | "reset" | "clean" => Ok(CommandPolicy {
            action_class: if matches!(subcommand, "reset" | "clean" | "restore") {
                "destructive".to_string()
            } else {
                "workspace_mutation".to_string()
            },
            risk_class: if matches!(subcommand, "reset" | "clean" | "restore" | "push") {
                crate::kernel::RiskClass::High
            } else {
                crate::kernel::RiskClass::Medium
            },
            requires_approval: true,
        }),
        _ => Err(format!("Unsupported git subcommand '{}'", subcommand)),
    }
}

fn classify_cargo(args: &[String]) -> std::result::Result<CommandPolicy, String> {
    let subcommand = args.first().map(String::as_str).unwrap_or("check");
    match subcommand {
        "check" | "test" | "build" | "fmt" | "clippy" | "doc" => Ok(CommandPolicy {
            action_class: "build".to_string(),
            risk_class: crate::kernel::RiskClass::Low,
            requires_approval: false,
        }),
        "run" | "install" | "add" | "remove" | "update" => Ok(CommandPolicy {
            action_class: "workspace_mutation".to_string(),
            risk_class: crate::kernel::RiskClass::Medium,
            requires_approval: true,
        }),
        _ => Err(format!("Unsupported cargo subcommand '{}'", subcommand)),
    }
}

fn classify_node_pkg(
    program: &str,
    args: &[String],
) -> std::result::Result<CommandPolicy, String> {
    let subcommand = args.first().map(String::as_str).unwrap_or("test");
    match (program, subcommand) {
        ("npm", "test") | ("pnpm", "test") | ("yarn", "test") => Ok(CommandPolicy {
            action_class: "build".to_string(),
            risk_class: crate::kernel::RiskClass::Low,
            requires_approval: false,
        }),
        (_, "install") | (_, "add") | (_, "remove") | (_, "upgrade") | (_, "dlx") => {
            Ok(CommandPolicy {
                action_class: "workspace_mutation".to_string(),
                risk_class: crate::kernel::RiskClass::Medium,
                requires_approval: true,
            })
        }
        _ => Err(format!(
            "Unsupported {} subcommand '{}'",
            program, subcommand
        )),
    }
}

fn contains_shell_metacharacters(command: &str) -> bool {
    ["&&", "||", ";", "|", "$(", "`", ">", "<"]
        .iter()
        .any(|pattern| command.contains(pattern))
}

fn missing_argument(tool_name: &str, argument: &str) -> ToolResult {
    ToolResult {
        tool_name: tool_name.to_string(),
        success: false,
        output: String::new(),
        error: Some(format!("Missing '{}' argument", argument)),
        requires_approval: false,
        approval_id: None,
        metadata: None,
    }
}

fn error_result(tool_name: &str, error: crate::OraError) -> ToolResult {
    ToolResult {
        tool_name: tool_name.to_string(),
        success: false,
        output: String::new(),
        error: Some(error.to_string()),
        requires_approval: false,
        approval_id: None,
        metadata: None,
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
}

fn memory_info() -> u64 {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|contents| {
                contents
                    .lines()
                    .find(|line| line.starts_with("MemAvailable:"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|value| value.parse::<u64>().ok())
            })
            .map(|value| value / 1024)
            .unwrap_or(0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

fn disk_info() -> String {
    #[cfg(target_os = "linux")]
    {
        Command::new("df")
            .args(["-h", "."])
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_shell_metacharacters() {
        assert!(contains_shell_metacharacters("git status && git diff"));
        assert!(!contains_shell_metacharacters("git status"));
    }

    #[test]
    fn test_classify_git() {
        assert!(classify_command("git", &[String::from("status")]).is_ok());
        let policy = classify_command("git", &[String::from("push")]).expect("git push policy");
        assert!(policy.requires_approval);
    }

    #[test]
    fn test_classify_rm_requires_approval() {
        let policy = classify_command("rm", &[String::from("-rf"), String::from("tmp")])
            .expect("rm policy");
        assert!(policy.requires_approval);
    }
}
