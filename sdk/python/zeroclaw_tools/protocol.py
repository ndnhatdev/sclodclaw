"""Stable public protocol seam for app/client integrations."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal, TypedDict

PROTOCOL_VERSION: Literal["redclaw.v1"] = "redclaw.v1"
WS_SUBPROTOCOL: Literal["redclaw.v1"] = PROTOCOL_VERSION

PUBLIC_HEALTH_PATH = "/health"
PAIR_PATH = "/pair"
EVENTS_STREAM_PATH = "/api/events"
WS_CHAT_PATH = "/ws/chat"

ProtocolErrorCode = Literal[
    "validation_error",
    "auth_error",
    "policy_denied",
    "runtime_unavailable",
    "internal_error",
]


@dataclass(frozen=True)
class ProtocolError:
    """Stable protocol error payload."""

    code: ProtocolErrorCode
    message: str


class ProtocolSessionHandle(TypedDict, total=False):
    """Stable protocol session handle."""

    session_id: str
    version: Literal["redclaw.v1"]


class CreateSessionCommand(TypedDict):
    type: Literal["create_session"]


class ResumeSessionCommand(TypedDict):
    type: Literal["resume_session"]
    handle: ProtocolSessionHandle


class CloseSessionCommand(TypedDict):
    type: Literal["close_session"]
    handle: ProtocolSessionHandle


class SubmitTurnCommand(TypedDict, total=False):
    type: Literal["submit_turn"]
    content: str
    handle: ProtocolSessionHandle


class MessageCommand(TypedDict):
    type: Literal["message"]
    content: str


ClientCommand = (
    CreateSessionCommand
    | ResumeSessionCommand
    | CloseSessionCommand
    | SubmitTurnCommand
    | MessageCommand
)


class ClientEvent(TypedDict, total=False):
    """Minimal event shape shared by WebSocket and SSE flows."""

    type: str
    content: str
    full_response: str
    message: str
    code: ProtocolErrorCode
    component: str
    provider: str
    model: str
    name: str
    tool: str
    args: object
    output: str
    success: bool
    duration_ms: int
    timestamp: str
    tokens_used: int
    cost_usd: float


def message_command(content: str) -> MessageCommand:
    """Build a backward-compatible single-turn message command."""

    return {
        "type": "message",
        "content": content,
    }


def submit_turn_command(
    content: str,
    handle: ProtocolSessionHandle | None = None,
) -> SubmitTurnCommand:
    """Build a session-aware submit-turn command."""

    command: SubmitTurnCommand = {
        "type": "submit_turn",
        "content": content,
    }
    if handle is not None:
        command["handle"] = handle
    return command
