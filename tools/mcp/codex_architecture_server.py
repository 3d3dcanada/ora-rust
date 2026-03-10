#!/usr/bin/env python3
from typing import Any

from codex_cli_server import _run_codex
from jsonrpc_stdio import StdioMcpServer


TOOLS = [
    {
        "name": "analyze_system",
        "description": "Analyze a system, subsystem, or module and describe structure, flows, and constraints.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "question": {"type": "string"},
                "cwd": {"type": "string", "default": "."},
            },
            "required": ["question"],
        },
    },
    {
        "name": "plan_feature",
        "description": "Create a decision-ready implementation plan for a feature or refactor.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "feature_request": {"type": "string"},
                "constraints": {"type": "string", "default": ""},
                "cwd": {"type": "string", "default": "."},
            },
            "required": ["feature_request"],
        },
    },
    {
        "name": "review_risks",
        "description": "Review risks, regressions, missing tests, and rollout concerns for a proposed change.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "change_summary": {"type": "string"},
                "cwd": {"type": "string", "default": "."},
            },
            "required": ["change_summary"],
        },
    },
    {
        "name": "migration_outline",
        "description": "Outline a migration path between current and target architecture or tooling.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "source_state": {"type": "string"},
                "target_state": {"type": "string"},
                "constraints": {"type": "string", "default": ""},
                "cwd": {"type": "string", "default": "."},
            },
            "required": ["source_state", "target_state"],
        },
    },
    {
        "name": "acceptance_checklist",
        "description": "Generate acceptance criteria, edge cases, and verification steps for a change.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "feature_request": {"type": "string"},
                "cwd": {"type": "string", "default": "."},
            },
            "required": ["feature_request"],
        },
    },
]


def _call(prompt: str, cwd: str) -> dict[str, Any]:
    text = _run_codex(prompt=prompt, cwd=cwd, sandbox="read-only")
    return {"content": [{"type": "text", "text": text}], "isError": False}


def handle_tool(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    cwd = str(arguments.get("cwd", "."))
    if name == "analyze_system":
        prompt = (
            "Analyze the system or module described below.\n"
            f"Question: {arguments['question']}\n"
            "Explain architecture, responsibilities, dependencies, failure points, and implementation constraints."
        )
        return _call(prompt, cwd)

    if name == "plan_feature":
        prompt = (
            "Produce a decision-complete implementation plan.\n"
            f"Feature request: {arguments['feature_request']}\n"
            f"Constraints: {arguments.get('constraints', '')}\n"
            "Include interfaces, data flow, edge cases, tests, and migration concerns."
        )
        return _call(prompt, cwd)

    if name == "review_risks":
        prompt = (
            "Perform a code-review style risk analysis.\n"
            f"Change summary: {arguments['change_summary']}\n"
            "Prioritize bugs, regressions, unsafe assumptions, and missing tests."
        )
        return _call(prompt, cwd)

    if name == "migration_outline":
        prompt = (
            "Outline a migration strategy.\n"
            f"Current state: {arguments['source_state']}\n"
            f"Target state: {arguments['target_state']}\n"
            f"Constraints: {arguments.get('constraints', '')}\n"
            "Return phased steps, compatibility strategy, rollback points, and verification."
        )
        return _call(prompt, cwd)

    if name == "acceptance_checklist":
        prompt = (
            "Generate an acceptance checklist for this feature.\n"
            f"Feature request: {arguments['feature_request']}\n"
            "Include happy path, edge cases, regression checks, observability, and rollout validation."
        )
        return _call(prompt, cwd)

    return {"content": [{"type": "text", "text": f"Unknown tool: {name}"}], "isError": True}


if __name__ == "__main__":
    StdioMcpServer("codex-architecture-local", "0.1.0", TOOLS, handle_tool).run()
