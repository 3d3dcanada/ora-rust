//! OrA Tool Definitions
//!
//! Tools available to the LLM for agentic operations.

use serde::{Deserialize, Serialize};

/// Tool definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool type (always "function" for our use)
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Function definition
    pub function: FunctionDefinition,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(name: &str, description: &str, parameters: serde_json::Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: name.to_string(),
                description: description.to_string(),
                parameters,
            },
        }
    }
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Get all available tools for OrA
pub fn get_ora_tools() -> Vec<ToolDefinition> {
    vec![
        // File operations
        ToolDefinition::new(
            "read_file",
            "Read contents of a file from the filesystem",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        ),
        ToolDefinition::new(
            "write_file",
            "Write content to a file in the workspace",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path where to write the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        ),
        ToolDefinition::new(
            "list_directory",
            "List contents of a directory",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory to list"
                    }
                },
                "required": ["path"]
            }),
        ),
        // Command execution
        ToolDefinition::new(
            "execute_command",
            "Execute a shell command in the workspace",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Working directory for the command (optional)"
                    }
                },
                "required": ["command"]
            }),
        ),
        // Web search
        ToolDefinition::new(
            "web_search",
            "Search the web for information",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "num_results": {
                        "type": "integer",
                        "description": "Number of results to return (default 5)",
                        "default": 5
                    }
                },
                "required": ["query"]
            }),
        ),
        // Code analysis
        ToolDefinition::new(
            "analyze_code",
            "Analyze code for issues, bugs, or improvements",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code to analyze"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language of the code"
                    },
                    "analysis_type": {
                        "type": "string",
                        "description": "Type of analysis (bugs, security, style, general)",
                        "enum": ["bugs", "security", "style", "general"]
                    }
                },
                "required": ["code", "language"]
            }),
        ),
        // System info
        ToolDefinition::new(
            "get_system_info",
            "Get information about the system",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "info_type": {
                        "type": "string",
                        "description": "Type of system info to retrieve",
                        "enum": ["cpu", "memory", "disk", "network", "all"]
                    }
                },
                "required": ["info_type"]
            }),
        ),
        // Memory/Context
        ToolDefinition::new(
            "search_memory",
            "Search the agent's memory for relevant context",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "default": 5
                    }
                },
                "required": ["query"]
            }),
        ),
    ]
}
