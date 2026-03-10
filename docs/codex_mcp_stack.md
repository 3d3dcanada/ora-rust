# Codex MCP Stack

This repo now contains the local pieces for the zero-cost Codex MCP stack:

- `brave`: local Brave Search MCP wrapper. Requires `BRAVE_API_KEY`.
- `claude`: `claude mcp serve`.
- `linear`: official remote MCP at `https://mcp.linear.app/mcp`.
- `playwright`: `@playwright/mcp`.
- `context7`: official remote MCP at `https://mcp.context7.com/mcp`.
- `codex_cli`: local Codex task wrapper.
- `codex_architecture`: local Codex architecture/planning wrapper.
- `browsermcp`: `@browsermcp/mcp`.
- `sequential_thinking`: `@modelcontextprotocol/server-sequential-thinking`.
- `openai_docs`: official OpenAI Docs MCP at `https://developers.openai.com/mcp`.

Local files added in this repo:

- `tools/mcp/jsonrpc_stdio.py`
- `tools/mcp/brave_search_server.py`
- `tools/mcp/codex_cli_server.py`
- `tools/mcp/codex_architecture_server.py`
- `scripts/install_codex_mcp_stack.py`

Install the stack into Codex config:

```bash
python3 scripts/install_codex_mcp_stack.py
```

Required env vars and auth:

- `BRAVE_API_KEY` for the local Brave MCP server.
- `codex mcp login linear` to complete Linear auth after config is written.
- `BrowserMCP` requires its browser extension to be installed and connected.
- `Context7` works without auth at basic limits; add an API key later only if you want higher rate limits.

Recommended smoke checks:

```bash
codex mcp list
codex mcp get brave
codex mcp get codex_cli
codex mcp get openai_docs
```
