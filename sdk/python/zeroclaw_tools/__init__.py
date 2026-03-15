"""
RedClaw Tools - LangGraph-based tool calling for consistent LLM agent execution.

This package provides a reliable tool-calling layer for LLM providers that may have
inconsistent native tool calling behavior. Built on LangGraph for guaranteed execution.
"""

from .agent import create_agent, ZeroclawAgent
from .client import ClientSdk
from .protocol import (
    ClientCommand,
    ClientEvent,
    EVENTS_STREAM_PATH,
    MessageCommand,
    PAIR_PATH,
    PROTOCOL_VERSION,
    ProtocolErrorCode,
    ProtocolSessionHandle,
    PUBLIC_HEALTH_PATH,
    SubmitTurnCommand,
    WS_CHAT_PATH,
    WS_SUBPROTOCOL,
    ProtocolError,
    message_command,
    submit_turn_command,
)
from .tools import (
    shell,
    file_read,
    file_write,
    web_search,
    http_request,
    memory_store,
    memory_recall,
)
from .tools.base import tool  # pyright: ignore[reportUnknownVariableType]

__version__ = "0.1.0"
__all__ = [
    "create_agent",
    "ZeroclawAgent",
    "ClientSdk",
    "PROTOCOL_VERSION",
    "WS_SUBPROTOCOL",
    "ProtocolErrorCode",
    "ProtocolSessionHandle",
    "ClientCommand",
    "ClientEvent",
    "PUBLIC_HEALTH_PATH",
    "PAIR_PATH",
    "EVENTS_STREAM_PATH",
    "WS_CHAT_PATH",
    "ProtocolError",
    "MessageCommand",
    "SubmitTurnCommand",
    "message_command",
    "submit_turn_command",
    "tool",
    "shell",
    "file_read",
    "file_write",
    "web_search",
    "http_request",
    "memory_store",
    "memory_recall",
]
