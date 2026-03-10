# ORA: MCP Ecosystem Integration Plan (Late 2026)

## 1. The Ultimate Goal
For ORA to succeed, it cannot merely exist as an isolated API. It must become the **default, governed enterprise memory server** within the greater Model Context Protocol (MCP) ecosystem. When an enterprise user opens Claude Desktop or Cursor, ORA should be a 1-click install.

## 2. Integration Roadmap

### Phase 1: The Local MCP Server (Claude Desktop & Cursor)
1.  **Build the Standard I/O Adapter:** Ora-Rust must implement the JSON-RPC 2.0 standard over `stdio` so it can be invoked locally as an MCP server.
2.  **Claude configuration:** Provide a copy-paste JSON configuration for users to add to their `claude_desktop_config.json`.
    ```json
    {
      "mcpServers": {
        "ora-memory": {
          "command": "ora-rust",
          "args": ["serve", "--mcp", "--vault-path", "~/.ora/vault"]
        }
      }
    }
    ```
3.  **Cursor IDE:** Ensure the same adapter works seamlessly in Cursor's MCP extension tab. This captures the massive developer audience instantly.

### Phase 2: Open-Source Registry Domination
The MCP Official Registry (previewed late 2025, fully active 2026) is the App Store for MCP servers. ORA must be listed prominently.

1.  **Validation:** Use the official Anthropic MCP Inspector (`npx @modelcontextprotocol/inspector ora-rust ...`) to ensure 100% compliance.
2.  **Submission Process:**
    *   Create a dedicated `package.json` with the required metadata.
    *   Use the official MCP publisher CLI to authenticate via GitHub OAuth.
    *   Submit `ora-rust-mcp`.
    *   **Registry Description:** "Enterprise-grade, securely governed (A0-A5) AI memory and context server. Prevents prompt injection and audits all memory access."

### Phase 3: Unofficial Marketplaces & Hubs
Beyond the central registry, ORA must be listed everywhere else developers look for context tools.
*   **Submit to MCPTop (`mcptop.art`)**
*   **Submit to MCP Server Hub (`mcpserverhub.net`)**
*   **LangChain / LlamaIndex integrations:** While ORA is an MCP server, maintaining first-class `langchain-ora` and `llama-index-ora` wrapper packages ensures we don't ignore users still using those legacy orchestration systems.

## 3. The "Network Effect" Strategy
By acting as an MCP server, ORA becomes inherently plug-and-play. If a user already uses a "GitHub MCP" (to read code) and a "Postgres MCP" (to read databases), ORA slides in perfectly as the "Governed Memory MCP". 
Marketing hook: *"Don't let your Claude Desktop read your private codebase without an audit log. Funnel all MCP traffic through ORA's A5 Gates."* (ORA acting as an MCP Proxy/Router).
