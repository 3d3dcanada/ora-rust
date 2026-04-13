//! OrA Model Context Protocol (MCP) Server
//!
//! Exposes OrA's secure tools and context over stdio for IDE integration (Claude Desktop, Cursor, etc).
//! All tool calls must pass through the AST Parser security gates.

use crate::error::Result;
use crate::kernel::ToolExecutor;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

pub struct McpServer {
    state: AppState,
    tools: Arc<ToolExecutor>,
}

impl McpServer {
    pub fn new(state: AppState) -> Self {
        let tools = Arc::new(ToolExecutor::new(state.clone()));
        Self { state, tools }
    }

    /// Primary stdio event loop for MCP
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = stdin();
        let mut reader = BufReader::new(stdin).lines();
        let mut stdout = stdout();

        while let Ok(Some(line)) = reader.next_line().await {
            // Parse incoming JSON-RPC
            let req: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    let out = serde_json::to_string(&err_resp).unwrap() + "\n";
                    let _ = stdout.write_all(out.as_bytes()).await;
                    continue;
                }
            };

            // Handle the request
            let res = self.handle_request(&req).await;

            // Write response
            if let Ok(json_out) = serde_json::to_string(&res) {
                let out = json_out + "\n";
                let _ = stdout.write_all(out.as_bytes()).await;
                let _ = stdout.flush().await; // Ensure swift delivery
            }
        }

        Ok(())
    }

    /// Handle a single MCP JSON-RPC request, useful for HTTP transport.
    pub async fn handle_jsonrpc(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        self.handle_request(&req).await
    }

    async fn handle_request(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id.clone();

        let result = match req.method.as_str() {
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => {
                self.handle_tools_call(req.params.clone().unwrap_or_default())
                    .await
            }
            "resources/list" => self.handle_resources_list().await,
            "resources/read" => self.handle_resources_read(&req.params).await,
            "initialize" => Ok(serde_json::json!({
                "protocolVersion": "2024-11-05", // Standard MCP spec version
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    },
                    "resources": {
                        "subscribe": false,
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": "ora-rust-secure-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            _ => Err(JsonRpcError {
                code: -32601,
                message: "Method not found".into(),
                data: None,
            }),
        };

        match result {
            Ok(val) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(val),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(e),
            },
        }
    }

    async fn handle_tools_list(&self) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let tools = vec![
            serde_json::json!({
                "name": "verified_answer",
                "description": "Answers a question with route tracing, evidence persistence, and grounded citations.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "query": { "type": "string" } },
                    "required": ["query"]
                }
            }),
            serde_json::json!({
                "name": "grounded_summarize",
                "description": "Summarizes provided content and stores the summary as a grounded evidence bundle.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "text": { "type": "string" }
                    },
                    "required": ["text"]
                }
            }),
            serde_json::json!({
                "name": "memory_search",
                "description": "Searches durable ORA memory records with provenance and confidence.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "default": 5 }
                    },
                    "required": ["query"]
                }
            }),
            serde_json::json!({
                "name": "safe_browser_task",
                "description": "Registers a governed browser task and creates approvals for sensitive actions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task": { "type": "string" },
                        "url": { "type": "string" },
                        "allowed_domains": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "required": ["task"]
                }
            }),
            serde_json::json!({
                "name": "approval_queue",
                "description": "Lists pending ORA approval requests.",
                "inputSchema": { "type": "object", "properties": {} }
            }),
            serde_json::json!({
                "name": "evidence_bundle",
                "description": "Retrieves a stored evidence bundle by id.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "bundle_id": { "type": "string" }
                    },
                    "required": ["bundle_id"]
                }
            }),
            serde_json::json!({
                "name": "create_mission",
                "description": "Creates a repeatable research mission with sources, extraction rules, and freshness policy.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "query": { "type": "string" },
                        "sources": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "extraction_rules": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "freshness_policy": { "type": "string" }
                    },
                    "required": ["name", "query"]
                }
            }),
            serde_json::json!({
                "name": "list_missions",
                "description": "Lists stored ORA research missions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer", "default": 10 }
                    }
                }
            }),
        ];

        Ok(serde_json::json!({
            "tools": tools
        }))
    }

    async fn handle_tools_call(
        &self,
        params: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or_default();

        // 1. SECURITY GATES CHECK (Crucial MCP Requirement)
        // Serialize arguments to check for prompt injection or malicious payloads targeting OrA
        let payload_str = serde_json::to_string(&arguments).unwrap_or_default();
        let gate_result = self.state.security_gates.parse_prompt(&payload_str);

        if !gate_result.passed {
            return Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": format!("SECURITY VIOLATION BLOCKED: {:?}", gate_result.reason)
                    }
                ],
                "isError": true
            }));
        }

        // 2. Execute Tool Safely
        let result = self.tools.execute(name, arguments).await;

        if result.success {
            Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": result.output
                    }
                ],
                "metadata": result.metadata,
                "approvalId": result.approval_id,
                "requiresApproval": result.requires_approval,
                "isError": false
            }))
        } else {
            Ok(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": result.error.unwrap_or_else(|| "Unknown error".to_string())
                    }
                ],
                "metadata": result.metadata,
                "approvalId": result.approval_id,
                "requiresApproval": result.requires_approval,
                "isError": true
            }))
        }
    }

    async fn handle_resources_list(&self) -> std::result::Result<serde_json::Value, JsonRpcError> {
        Ok(serde_json::json!({ "resources": [] }))
    }

    async fn handle_resources_read(
        &self,
        _params: &Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, JsonRpcError> {
        Err(JsonRpcError {
            code: -32601,
            message: "No resources implemented".into(),
            data: None,
        })
    }
}
