#!/usr/bin/env python3
import datetime as dt
import os
import shutil
from pathlib import Path

import tomllib


ROOT = Path(__file__).resolve().parents[1]
CONFIG_PATH = Path.home() / ".codex" / "config.toml"


def toml_value(value, indent=""):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        escaped = value.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(item, indent) for item in value) + "]"
    if isinstance(value, dict):
        inner = ", ".join(f"{key} = {toml_value(val, indent)}" for key, val in value.items())
        return "{ " + inner + " }"
    raise TypeError(f"Unsupported TOML value: {type(value)!r}")


def emit_table(lines, prefix, table):
    scalar_items = []
    nested_items = []
    for key, value in table.items():
        if isinstance(value, dict):
            nested_items.append((key, value))
        else:
            scalar_items.append((key, value))

    if prefix:
        lines.append(f"[{prefix}]")
    for key, value in scalar_items:
        lines.append(f"{key} = {toml_value(value)}")
    if prefix and (scalar_items or nested_items):
        lines.append("")

    for idx, (key, value) in enumerate(nested_items):
        child_prefix = f"{prefix}.{key}" if prefix else key
        emit_table(lines, child_prefix, value)
        if idx != len(nested_items) - 1:
            lines.append("")


def main():
    CONFIG_PATH.parent.mkdir(parents=True, exist_ok=True)
    existing = {}
    if CONFIG_PATH.exists():
        existing = tomllib.loads(CONFIG_PATH.read_text(encoding="utf-8"))
        backup = CONFIG_PATH.with_suffix(f".bak.{dt.datetime.now().strftime('%Y%m%d%H%M%S')}")
        shutil.copy2(CONFIG_PATH, backup)
        print(f"Backed up existing config to {backup}")

    features = dict(existing.get("features", {}))
    features["rmcp_client"] = True
    features["experimental_use_rmcp_client"] = True
    existing["features"] = features

    mcp_servers = dict(existing.get("mcp_servers", {}))
    mcp_servers["playwright"] = {
        "command": "npx",
        "args": ["-y", "@playwright/mcp@latest"],
    }
    mcp_servers["figma"] = existing.get("mcp_servers", {}).get("figma", {"url": "https://mcp.figma.com/mcp"})
    mcp_servers["brave"] = {
        "command": "python3",
        "args": [str(ROOT / "tools/mcp/brave_search_server.py")],
    }
    mcp_servers["claude"] = {
        "command": "claude",
        "args": ["mcp", "serve"],
    }
    mcp_servers["linear"] = {
        "url": "https://mcp.linear.app/mcp",
    }
    mcp_servers["context7"] = {
        "url": "https://mcp.context7.com/mcp",
    }
    mcp_servers["codex_cli"] = {
        "command": "python3",
        "args": [str(ROOT / "tools/mcp/codex_cli_server.py")],
    }
    mcp_servers["codex_architecture"] = {
        "command": "python3",
        "args": [str(ROOT / "tools/mcp/codex_architecture_server.py")],
    }
    mcp_servers["browsermcp"] = {
        "command": "npx",
        "args": ["-y", "@browsermcp/mcp@latest"],
    }
    mcp_servers["sequential_thinking"] = {
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
    }
    mcp_servers["openai_docs"] = {
        "url": "https://developers.openai.com/mcp",
    }
    existing["mcp_servers"] = mcp_servers

    lines: list[str] = []
    emit_table(lines, "", existing)
    content = "\n".join(line for line in lines if line is not None).strip() + "\n"
    CONFIG_PATH.write_text(content, encoding="utf-8")
    print(f"Wrote {CONFIG_PATH}")


if __name__ == "__main__":
    main()
