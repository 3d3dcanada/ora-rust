#!/usr/bin/env python3
import json
import sys
import traceback
from typing import Any, Callable


class StdioMcpServer:
    def __init__(self, name: str, version: str, tools: list[dict[str, Any]], handler: Callable[[str, dict[str, Any]], Any]):
        self.name = name
        self.version = version
        self.tools = tools
        self.handler = handler

    def _response(self, request_id: Any, result: Any = None, error: dict[str, Any] | None = None) -> dict[str, Any]:
        payload: dict[str, Any] = {"jsonrpc": "2.0", "id": request_id}
        if error is not None:
            payload["error"] = error
        else:
            payload["result"] = result
        return payload

    def _write(self, message: dict[str, Any]) -> None:
        sys.stdout.write(json.dumps(message) + "\n")
        sys.stdout.flush()

    def _tool_result(self, text: str, is_error: bool = False) -> dict[str, Any]:
        return {
            "content": [{"type": "text", "text": text}],
            "isError": is_error,
        }

    def run(self) -> None:
        for raw_line in sys.stdin:
            line = raw_line.strip()
            if not line:
                continue
            try:
                request = json.loads(line)
                request_id = request.get("id")
                method = request.get("method")
                params = request.get("params") or {}

                if method == "initialize":
                    result = {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {"listChanged": False},
                            "resources": {"subscribe": False, "listChanged": False},
                        },
                        "serverInfo": {"name": self.name, "version": self.version},
                    }
                    if request_id is not None:
                        self._write(self._response(request_id, result=result))
                    continue

                if method == "notifications/initialized":
                    continue

                if method == "ping":
                    if request_id is not None:
                        self._write(self._response(request_id, result={}))
                    continue

                if method == "tools/list":
                    if request_id is not None:
                        self._write(self._response(request_id, result={"tools": self.tools}))
                    continue

                if method == "resources/list":
                    if request_id is not None:
                        self._write(self._response(request_id, result={"resources": []}))
                    continue

                if method == "tools/call":
                    tool_name = params.get("name", "")
                    arguments = params.get("arguments") or {}
                    result = self.handler(tool_name, arguments)
                    if isinstance(result, dict) and "content" in result:
                        payload = result
                    else:
                        payload = self._tool_result(str(result))
                    if request_id is not None:
                        self._write(self._response(request_id, result=payload))
                    continue

                if request_id is not None:
                    self._write(
                        self._response(
                            request_id,
                            error={"code": -32601, "message": f"Method not found: {method}"},
                        )
                    )
            except Exception as exc:
                if "request_id" in locals() and request_id is not None:
                    self._write(
                        self._response(
                            request_id,
                            error={
                                "code": -32000,
                                "message": str(exc),
                                "data": traceback.format_exc(),
                            },
                        )
                    )
                else:
                    sys.stderr.write(traceback.format_exc() + "\n")
