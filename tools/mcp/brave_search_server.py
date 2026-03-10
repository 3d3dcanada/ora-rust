#!/usr/bin/env python3
import json
import os
import re
import urllib.parse
import urllib.request
from typing import Any

from jsonrpc_stdio import StdioMcpServer


TOOLS = [
    {
        "name": "web_search",
        "description": "Search the web with Brave Search and return structured SERP results.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "count": {"type": "integer", "minimum": 1, "maximum": 20, "default": 8},
                "country": {"type": "string", "default": "us"},
                "search_lang": {"type": "string", "default": "en"},
            },
            "required": ["query"],
        },
    },
    {
        "name": "discover_content_angles",
        "description": "Use live Brave SERP data to suggest content angles, titles, and competitor patterns.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "topic": {"type": "string"},
                "audience": {"type": "string", "default": ""},
                "count": {"type": "integer", "minimum": 3, "maximum": 12, "default": 8},
            },
            "required": ["topic"],
        },
    },
]


def _require_api_key() -> str:
    api_key = os.environ.get("BRAVE_API_KEY")
    if not api_key:
        raise RuntimeError("BRAVE_API_KEY is not set")
    return api_key


def _search(query: str, count: int, country: str, search_lang: str) -> dict[str, Any]:
    api_key = _require_api_key()
    params = urllib.parse.urlencode(
        {
            "q": query,
            "count": max(1, min(count, 20)),
            "country": country,
            "search_lang": search_lang,
            "text_decorations": "false",
            "result_filter": "web,query",
        }
    )
    request = urllib.request.Request(
        f"https://api.search.brave.com/res/v1/web/search?{params}",
        headers={
            "Accept": "application/json",
            "X-Subscription-Token": api_key,
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read().decode("utf-8"))


def _format_results(payload: dict[str, Any]) -> str:
    web_results = payload.get("web", {}).get("results", [])
    query_info = payload.get("query", {})
    lines = [f"Query: {query_info.get('original', '')}", f"Results returned: {len(web_results)}", ""]
    for index, item in enumerate(web_results, start=1):
        title = item.get("title", "").strip()
        url = item.get("url", "").strip()
        description = re.sub(r"\s+", " ", item.get("description", "").strip())
        age = item.get("age")
        lines.append(f"{index}. {title}")
        lines.append(f"   URL: {url}")
        if age:
            lines.append(f"   Age: {age}")
        if description:
            lines.append(f"   Summary: {description}")
        lines.append("")
    return "\n".join(lines).strip()


def _extract_angles(payload: dict[str, Any], topic: str, audience: str) -> str:
    web_results = payload.get("web", {}).get("results", [])
    title_words: dict[str, int] = {}
    patterns: list[str] = []
    for item in web_results:
        title = item.get("title", "")
        words = re.findall(r"[A-Za-z0-9][A-Za-z0-9+/-]{2,}", title.lower())
        for word in words:
            if word in {"with", "that", "from", "your", "this", "into", "what", "when", "where"}:
                continue
            title_words[word] = title_words.get(word, 0) + 1
        if title:
            patterns.append(title)

    common_terms = sorted(title_words.items(), key=lambda item: (-item[1], item[0]))[:8]
    angle_lines = [f"Topic: {topic}"]
    if audience:
        angle_lines.append(f"Audience: {audience}")
    angle_lines.append("")
    angle_lines.append("SERP patterns:")
    for pattern in patterns[:6]:
        angle_lines.append(f"- {pattern}")
    angle_lines.append("")
    angle_lines.append("Repeated terms:")
    for term, score in common_terms:
        angle_lines.append(f"- {term}: {score}")
    angle_lines.append("")
    angle_lines.append("Suggested content angles:")
    suggested_angles = [
        f"{topic}: best-practice breakdown for {audience or 'operators and builders'}",
        f"{topic}: competitor teardown and market gaps",
        f"{topic}: tactical implementation checklist",
        f"{topic}: cost, ROI, and workflow comparison",
        f"{topic}: 2026 update and what changed recently",
    ]
    for angle in suggested_angles:
        angle_lines.append(f"- {angle}")
    return "\n".join(angle_lines)


def handle_tool(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    if name == "web_search":
        payload = _search(
            query=str(arguments["query"]),
            count=int(arguments.get("count", 8)),
            country=str(arguments.get("country", "us")),
            search_lang=str(arguments.get("search_lang", "en")),
        )
        return {"content": [{"type": "text", "text": _format_results(payload)}], "isError": False}

    if name == "discover_content_angles":
        payload = _search(
            query=str(arguments["topic"]),
            count=int(arguments.get("count", 8)),
            country="us",
            search_lang="en",
        )
        text = _extract_angles(payload, str(arguments["topic"]), str(arguments.get("audience", "")))
        return {"content": [{"type": "text", "text": text}], "isError": False}

    return {"content": [{"type": "text", "text": f"Unknown tool: {name}"}], "isError": True}


if __name__ == "__main__":
    StdioMcpServer("brave-search-local", "0.1.0", TOOLS, handle_tool).run()
