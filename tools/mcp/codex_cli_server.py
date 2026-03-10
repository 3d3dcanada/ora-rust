#!/usr/bin/env python3
import os
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from jsonrpc_stdio import StdioMcpServer


TOOLS = [
    {
        "name": "run_task",
        "description": "Run a repo-aware Codex task and return the final response.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "prompt": {"type": "string"},
                "cwd": {"type": "string", "default": "."},
                "sandbox": {"type": "string", "enum": ["read-only", "workspace-write"], "default": "workspace-write"},
                "model": {"type": "string"},
            },
            "required": ["prompt"],
        },
    },
    {
        "name": "explain_repo_area",
        "description": "Explain a code area, module, or file with concrete implementation detail.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "question": {"type": "string", "default": "Explain how this area works and what depends on it."},
                "cwd": {"type": "string", "default": "."},
            },
            "required": ["path"],
        },
    },
    {
        "name": "generate_patch_plan",
        "description": "Ask Codex for a concrete implementation plan without editing files.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task": {"type": "string"},
                "cwd": {"type": "string", "default": "."},
            },
            "required": ["task"],
        },
    },
    {
        "name": "handoff_summary",
        "description": "Generate a concise handoff summary for the current repo state or a focused area.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "focus": {"type": "string", "default": "Summarize the current repo state, important files, and likely next steps."},
                "cwd": {"type": "string", "default": "."},
            },
        },
    },
]


def _workspace(path: str) -> str:
    return str(Path(path).resolve())


def _run_codex(prompt: str, cwd: str, sandbox: str, model: str | None = None) -> str:
    with tempfile.NamedTemporaryFile(prefix="codex-mcp-", suffix=".txt", delete=False) as output_file:
        output_path = output_file.name
    cmd = [
        "codex",
        "exec",
        "-C",
        _workspace(cwd),
        "-s",
        sandbox,
        "--skip-git-repo-check",
        "--output-last-message",
        output_path,
        prompt,
    ]
    if model:
        cmd[2:2] = ["-m", model]
    env = os.environ.copy()
    completed = subprocess.run(cmd, capture_output=True, text=True, timeout=1800, env=env)
    stdout = completed.stdout.strip()
    stderr = completed.stderr.strip()
    try:
        final_text = Path(output_path).read_text(encoding="utf-8").strip()
    finally:
        Path(output_path).unlink(missing_ok=True)
    if completed.returncode != 0:
        raise RuntimeError(f"codex exec failed with code {completed.returncode}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}")
    return final_text or stdout or "(Codex returned no final message)"


def handle_tool(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    if name == "run_task":
        text = _run_codex(
            prompt=str(arguments["prompt"]),
            cwd=str(arguments.get("cwd", ".")),
            sandbox=str(arguments.get("sandbox", "workspace-write")),
            model=arguments.get("model"),
        )
        return {"content": [{"type": "text", "text": text}], "isError": False}

    if name == "explain_repo_area":
        prompt = (
            f"Explain the repo area at `{arguments['path']}`.\n"
            f"Question: {arguments.get('question', 'Explain how this area works and what depends on it.')}\n"
            "Be concrete and reference implementation details."
        )
        text = _run_codex(prompt=prompt, cwd=str(arguments.get("cwd", ".")), sandbox="read-only")
        return {"content": [{"type": "text", "text": text}], "isError": False}

    if name == "generate_patch_plan":
        prompt = (
            "Produce an implementation plan only. Do not edit files.\n"
            f"Task: {arguments['task']}\n"
            "Return the plan in compact actionable bullets with risks and tests."
        )
        text = _run_codex(prompt=prompt, cwd=str(arguments.get("cwd", ".")), sandbox="read-only")
        return {"content": [{"type": "text", "text": text}], "isError": False}

    if name == "handoff_summary":
        prompt = (
            "Write a concise engineer-to-engineer handoff summary.\n"
            f"Focus: {arguments.get('focus', 'Summarize the current repo state, important files, and likely next steps.')}\n"
            "Include current state, likely risks, and next steps."
        )
        text = _run_codex(prompt=prompt, cwd=str(arguments.get("cwd", ".")), sandbox="read-only")
        return {"content": [{"type": "text", "text": text}], "isError": False}

    return {"content": [{"type": "text", "text": f"Unknown tool: {name}"}], "isError": True}


if __name__ == "__main__":
    StdioMcpServer("codex-cli-local", "0.1.0", TOOLS, handle_tool).run()
