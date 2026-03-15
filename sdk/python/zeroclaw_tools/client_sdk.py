"""Minimal client SDK helpers built on the public protocol seam."""

from __future__ import annotations

from dataclasses import dataclass
from urllib.parse import urlencode

from .protocol import (
    EVENTS_STREAM_PATH,
    PAIR_PATH,
    PUBLIC_HEALTH_PATH,
    WS_CHAT_PATH,
    WS_SUBPROTOCOL,
    MessageCommand,
    message_command,
)


@dataclass
class ClientSdk:
    """Utility helpers for client-side protocol URL and payload construction."""

    base_http_url: str = "http://localhost:5555"

    def __post_init__(self) -> None:
        normalized = self.base_http_url.strip().rstrip("/")
        self.base_http_url = normalized or "http://localhost:5555"

    @property
    def ws_subprotocol(self) -> str:
        """WebSocket sub-protocol token expected by the gateway."""

        return WS_SUBPROTOCOL

    def public_health_url(self) -> str:
        """Public health endpoint URL."""

        return f"{self.base_http_url}{PUBLIC_HEALTH_PATH}"

    def pair_url(self) -> str:
        """Pairing endpoint URL."""

        return f"{self.base_http_url}{PAIR_PATH}"

    def events_stream_url(self) -> str:
        """SSE event stream endpoint URL."""

        return f"{self.base_http_url}{EVENTS_STREAM_PATH}"

    def ws_chat_url(
        self,
        token: str | None = None,
        session_id: str | None = None,
    ) -> str:
        """WebSocket chat endpoint URL with optional auth/session query params."""

        ws_base = _http_to_ws_base(self.base_http_url)
        query: dict[str, str] = {}

        if token:
            query["token"] = token

        if session_id:
            query["session_id"] = session_id

        if query:
            return f"{ws_base}{WS_CHAT_PATH}?{urlencode(query)}"
        return f"{ws_base}{WS_CHAT_PATH}"

    def message_command(self, content: str) -> MessageCommand:
        """Build a backward-compatible message command payload."""

        return message_command(content)


def _http_to_ws_base(value: str) -> str:
    if value.startswith("https://"):
        return "wss://" + value[len("https://") :]
    if value.startswith("http://"):
        return "ws://" + value[len("http://") :]
    return value
