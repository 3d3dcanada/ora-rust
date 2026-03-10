//! OrA Tools Module
//!
//! Tool execution for filesystem, shell, and other operations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Tool executor
pub struct ToolExecutor {
    workspace_root: PathBuf,
}

impl ToolExecutor {
    /// Create new tool executor
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Execute a tool
    pub async fn execute(&self, tool_name: &str, arguments: serde_json::Value) -> ToolResult {
        match tool_name {
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
                error: Some(format!("Unknown tool: {}", tool_name)),
            },
        }
    }

    /// Read a file
    async fn read_file(&self, args: serde_json::Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).map(PathBuf::from);

        match path {
            Some(p) => {
                // Security: only allow files in workspace
                if !p.starts_with(&self.workspace_root) {
                    return ToolResult {
                        tool_name: "read_file".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some("Access denied: path outside workspace".to_string()),
                    };
                }

                match std::fs::read_to_string(&p) {
                    Ok(content) => ToolResult {
                        tool_name: "read_file".to_string(),
                        success: true,
                        output: content,
                        error: None,
                    },
                    Err(e) => ToolResult {
                        tool_name: "read_file".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                    },
                }
            }
            None => ToolResult {
                tool_name: "read_file".to_string(),
                success: false,
                output: String::new(),
                error: Some("Missing 'path' argument".to_string()),
            },
        }
    }

    /// Write a file
    async fn write_file(&self, args: serde_json::Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).map(PathBuf::from);
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from);

        match (path, content) {
            (Some(p), Some(c)) => {
                // Security: only allow files in workspace
                if !p.starts_with(&self.workspace_root) {
                    return ToolResult {
                        tool_name: "write_file".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some("Access denied: path outside workspace".to_string()),
                    };
                }

                // Create parent directories
                if let Some(parent) = p.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        return ToolResult {
                            tool_name: "write_file".to_string(),
                            success: false,
                            output: String::new(),
                            error: Some(e.to_string()),
                        };
                    }
                }

                match std::fs::write(&p, &c) {
                    Ok(_) => ToolResult {
                        tool_name: "write_file".to_string(),
                        success: true,
                        output: format!("Written {} bytes", c.len()),
                        error: None,
                    },
                    Err(e) => ToolResult {
                        tool_name: "write_file".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                    },
                }
            }
            _ => ToolResult {
                tool_name: "write_file".to_string(),
                success: false,
                output: String::new(),
                error: Some("Missing 'path' or 'content' argument".to_string()),
            },
        }
    }

    /// List directory
    async fn list_directory(&self, args: serde_json::Value) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).map(PathBuf::from);

        match path {
            Some(p) => {
                // Security: only allow in workspace
                if !p.starts_with(&self.workspace_root) && !p.starts_with("/") {
                    return ToolResult {
                        tool_name: "list_directory".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some("Access denied: path outside workspace".to_string()),
                    };
                }

                match std::fs::read_dir(&p) {
                    Ok(entries) => {
                        let mut output = String::new();
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let type_str = if path.is_dir() { "dir" } else { "file" };
                            output.push_str(&format!("{}: {}\n", type_str, path.display()));
                        }
                        ToolResult {
                            tool_name: "list_directory".to_string(),
                            success: true,
                            output,
                            error: None,
                        }
                    }
                    Err(e) => ToolResult {
                        tool_name: "list_directory".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                    },
                }
            }
            None => ToolResult {
                tool_name: "list_directory".to_string(),
                success: false,
                output: String::new(),
                error: Some("Missing 'path' argument".to_string()),
            },
        }
    }

    /// Execute shell command
    async fn execute_command(&self, args: serde_json::Value) -> ToolResult {
        let command = args.get("command").and_then(|v| v.as_str());

        match command {
            Some(cmd) => {
                // Security: Basic command validation
                let dangerous = ["rm -rf /", "dd if=", "mkfs", ":(){ :|:& };:"];
                for pattern in dangerous {
                    if cmd.contains(pattern) {
                        return ToolResult {
                            tool_name: "execute_command".to_string(),
                            success: false,
                            output: String::new(),
                            error: Some("Command blocked by security policy".to_string()),
                        };
                    }
                }

                // Execute command
                match std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(&self.workspace_root)
                    .output()
                {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        ToolResult {
                            tool_name: "execute_command".to_string(),
                            success: output.status.success(),
                            output: stdout.to_string(),
                            error: if stderr.is_empty() {
                                None
                            } else {
                                Some(stderr.to_string())
                            },
                        }
                    }
                    Err(e) => ToolResult {
                        tool_name: "execute_command".to_string(),
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                    },
                }
            }
            None => ToolResult {
                tool_name: "execute_command".to_string(),
                success: false,
                output: String::new(),
                error: Some("Missing 'command' argument".to_string()),
            },
        }
    }

    /// Web search (placeholder - would use actual API)
    async fn web_search(&self, args: serde_json::Value) -> ToolResult {
        let query = args.get("query").and_then(|v| v.as_str());

        match query {
            Some(q) => ToolResult {
                tool_name: "web_search".to_string(),
                success: true,
                output: format!("Web search placeholder for: {}", q),
                error: None,
            },
            None => ToolResult {
                tool_name: "web_search".to_string(),
                success: false,
                output: String::new(),
                error: Some("Missing 'query' argument".to_string()),
            },
        }
    }

    /// Analyze code
    async fn analyze_code(&self, args: serde_json::Value) -> ToolResult {
        let code = args.get("code").and_then(|v| v.as_str());

        match code {
            Some(c) => {
                // Simple static analysis
                let mut issues = Vec::new();

                // Check for common issues
                if c.contains("eval(") {
                    issues.push("Warning: use of eval() is dangerous");
                }
                if c.contains("exec(") {
                    issues.push("Warning: use of exec() requires caution");
                }
                if c.contains("password") && c.contains("=") {
                    issues.push("Warning: potential hardcoded password");
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
                }
            }
            None => ToolResult {
                tool_name: "analyze_code".to_string(),
                success: false,
                output: String::new(),
                error: Some("Missing 'code' argument".to_string()),
            },
        }
    }

    /// Get system info
    async fn get_system_info(&self, args: serde_json::Value) -> ToolResult {
        let info_type = args
            .get("info_type")
            .and_then(|v| v.as_str())
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
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
}

fn memory_info() -> usize {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemAvailable:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<usize>().ok())
            })
            .map(|k| k / 1024)
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
        std::fs::read_to_string("/proc/mounts")
            .ok()
            .map(|s| {
                s.lines()
                    .find(|l| l.starts_with("/ "))
                    .map(|l| l.split_whitespace().nth(1).unwrap_or("unknown").to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }
    #[cfg(not(target_os = "linux"))]
    {
        "unknown".to_string()
    }
}
