from __future__ import annotations

import asyncio
import enum
import json
import os
import sys
from dataclasses import dataclass
from typing import Any, AsyncIterator, cast

import structlog
from openai import OpenAI
from openai import (
    APIConnectionError,
    APITimeoutError,
    APIStatusError,
    AuthenticationError,
    BadRequestError,
    RateLimitError,
)
from google.protobuf import json_format, struct_pb2

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'generated'))
from generated import horbyta_service_pb2 as pb2

logger = structlog.get_logger(__name__)

_MAX_RETRIES = 3
_RETRY_BACKOFF_BASE = 1.5
_PROMPTS_DIR = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "prompts"))
_PROMPT_EN_PATH = os.path.join(_PROMPTS_DIR, "assistant_system_prompt.en.md")
_STREAM_END = object()


def _load_system_prompt(path: str) -> str:
    with open(path, "r", encoding="utf-8") as file:
        return file.read().strip()


def _next_stream_item(stream: Any) -> Any:
    return next(stream, _STREAM_END)


class ChunkKind(enum.Enum):
    TEXT_DELTA = "text_delta"
    TOOL_CALL = "tool_call"
    DONE = "done"


@dataclass
class OpenRouterChunk:
    kind: ChunkKind
    data: Any  # str for TEXT_DELTA | dict for TOOL_CALL | None for DONE


class OpenRouterRequestError(RuntimeError):
    def __init__(self, code: str, user_message: str, retryable: bool, details: dict[str, Any] | None = None) -> None:
        super().__init__(user_message)
        self.code = code
        self.user_message = user_message
        self.retryable = retryable
        self.details = details or {}


class OpenRouterClient:
    def __init__(self) -> None:
        self._base_url = os.environ.get("OPENROUTER_BASE_URL", "https://openrouter.ai/api/v1")
        self._site_url = os.environ.get("OPENROUTER_SITE_URL", "")
        self._app_title = os.environ.get("OPENROUTER_APP_NAME", "HormigaPM")
        self._system_prompt = _load_system_prompt(_PROMPT_EN_PATH)

    async def stream_turn(
        self,
        messages: list[pb2.Message],
        tools: list[pb2.ToolDefinition],
        model: str,
        api_key: str,
    ) -> AsyncIterator[OpenRouterChunk]:
        if not api_key:
            raise RuntimeError("Missing API key for model call")

        if not model:
            raise RuntimeError("Missing model for model call")

        payload_messages = self._to_chat_messages(messages)
        payload_tools = self._build_tools(tools) if tools else None

        for attempt in range(1, _MAX_RETRIES + 1):
            try:
                async for chunk in self._do_stream(
                    payload_messages,
                    payload_tools,
                    model,
                    api_key,
                ):
                    yield chunk
                return
            except Exception as exc:
                normalized = self._normalize_exception(exc)

                if not normalized.retryable:
                    logger.error(
                        "llm_stream_non_retryable_error",
                        error_code=normalized.code,
                        error=normalized.user_message,
                        model=model,
                        details=normalized.details,
                    )
                    raise normalized

                if attempt == _MAX_RETRIES:
                    logger.error(
                        "llm_stream_failed",
                        error_code=normalized.code,
                        error=normalized.user_message,
                        attempts=attempt,
                        model=model,
                        details=normalized.details,
                    )
                    raise normalized

                wait = _RETRY_BACKOFF_BASE ** attempt
                logger.warning(
                    "llm_stream_retrying",
                    error_code=normalized.code,
                    error=normalized.user_message,
                    attempt=attempt,
                    wait_seconds=wait,
                    model=model,
                    details=normalized.details,
                )
                await asyncio.sleep(wait)

    @staticmethod
    def _normalize_exception(exc: Exception) -> OpenRouterRequestError:
        if isinstance(exc, OpenRouterRequestError):
            return exc

        if isinstance(exc, AuthenticationError):
            return OpenRouterRequestError(
                code="OPENROUTER_AUTH",
                user_message="API key invalida o sin permisos en OpenRouter.",
                retryable=False,
            )

        if isinstance(exc, RateLimitError):
            return OpenRouterRequestError(
                code="OPENROUTER_RATE_LIMIT",
                user_message="OpenRouter aplico limite de tasa. Intenta en unos segundos.",
                retryable=True,
            )

        if isinstance(exc, APITimeoutError):
            return OpenRouterRequestError(
                code="OPENROUTER_TIMEOUT",
                user_message="OpenRouter tardo demasiado en responder.",
                retryable=True,
            )

        if isinstance(exc, APIConnectionError):
            return OpenRouterRequestError(
                code="OPENROUTER_CONNECTION",
                user_message="No se pudo conectar con OpenRouter.",
                retryable=True,
            )

        if isinstance(exc, BadRequestError):
            message = str(exc)
            lowered = message.lower()
            if "model" in lowered:
                return OpenRouterRequestError(
                    code="OPENROUTER_MODEL",
                    user_message="El modelo configurado no esta disponible en OpenRouter.",
                    retryable=False,
                )

            return OpenRouterRequestError(
                code="OPENROUTER_BAD_REQUEST",
                user_message="Solicitud invalida enviada a OpenRouter.",
                retryable=False,
                details={"raw": message},
            )

        if isinstance(exc, APIStatusError):
            status = exc.status_code
            retryable = status in {408, 429, 500, 502, 503, 504}
            return OpenRouterRequestError(
                code=f"OPENROUTER_HTTP_{status}",
                user_message=f"OpenRouter devolvio HTTP {status}.",
                retryable=retryable,
                details={"raw": str(exc)},
            )

        return OpenRouterRequestError(
            code="OPENROUTER_UNKNOWN",
            user_message="Error inesperado procesando respuesta del modelo.",
            retryable=False,
            details={"raw": str(exc)},
        )

    def _build_client(self, api_key: str) -> OpenAI:
        headers: dict[str, str] = {}
        if self._site_url:
            headers["HTTP-Referer"] = self._site_url
        if self._app_title:
            headers["X-OpenRouter-Title"] = self._app_title

        return OpenAI(
            api_key=api_key,
            base_url=self._base_url,
            default_headers=headers or None,
        )

    async def _do_stream(
        self,
        messages: list[dict[str, Any]],
        tools: list[dict[str, Any]] | None,
        model: str,
        api_key: str,
    ) -> AsyncIterator[OpenRouterChunk]:
        client = self._build_client(api_key)
        request_messages = [{"role": "system", "content": self._system_prompt}, *messages]
        request_payload: dict[str, Any] = {
            "model": model,
            "messages": request_messages,
            "stream": True,
            "parallel_tool_calls": False,
        }
        if tools:
            request_payload["tools"] = tools
            request_payload["tool_choice"] = "auto"

        stream = await asyncio.to_thread(client.chat.completions.create, **request_payload)
        tool_calls_by_index: dict[int, dict[str, str]] = {}

        while True:
            raw_chunk = await asyncio.to_thread(_next_stream_item, stream)
            if raw_chunk is _STREAM_END:
                break

            chunk = cast(Any, raw_chunk)
            choices = getattr(chunk, "choices", None) or []
            if not choices:
                continue

            delta = choices[0].delta
            if delta is None:
                continue

            if delta.content:
                yield OpenRouterChunk(kind=ChunkKind.TEXT_DELTA, data=delta.content)

            for tool_delta in delta.tool_calls or []:
                index = tool_delta.index or 0
                current = tool_calls_by_index.setdefault(index, {"id": "", "name": "", "arguments": ""})

                if tool_delta.id:
                    current["id"] = tool_delta.id

                fn_call = tool_delta.function
                if fn_call is None:
                    continue

                if fn_call.name:
                    current["name"] = fn_call.name

                if fn_call.arguments:
                    current["arguments"] += fn_call.arguments

        if tool_calls_by_index:
            first_key = sorted(tool_calls_by_index.keys())[0]
            first_call = tool_calls_by_index[first_key]

            try:
                args = json.loads(first_call["arguments"]) if first_call["arguments"] else {}
            except json.JSONDecodeError:
                args = {}

            yield OpenRouterChunk(
                kind=ChunkKind.TOOL_CALL,
                data={
                    "id": first_call["id"] or first_call["name"],
                    "name": first_call["name"],
                    "arguments": args,
                },
            )

        yield OpenRouterChunk(kind=ChunkKind.DONE, data=None)

    @staticmethod
    def _to_chat_messages(messages: list[pb2.Message]) -> list[dict[str, Any]]:
        payload: list[dict[str, Any]] = []

        for msg in messages:
            if msg.role == pb2.Message.Role.USER:
                if msg.content:
                    payload.append({"role": "user", "content": msg.content})
                continue

            if msg.role == pb2.Message.Role.ASSISTANT and msg.HasField("tool_call"):
                tc = msg.tool_call
                args_dict = OpenRouterClient._struct_to_dict(tc.arguments) or {}
                payload.append(
                    {
                        "role": "assistant",
                        "content": "",
                        "tool_calls": [
                            {
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": json.dumps(args_dict, ensure_ascii=True),
                                },
                            }
                        ],
                    }
                )
                continue

            if msg.role == pb2.Message.Role.SYSTEM and msg.HasField("tool_result"):
                tr = msg.tool_result
                result_dict = OpenRouterClient._struct_to_dict(tr.result) if tr.result else {}
                tool_output = {
                    "success": tr.success,
                    "result": result_dict,
                    "error_message": tr.error_message,
                }
                payload.append(
                    {
                        "role": "tool",
                        "tool_call_id": tr.tool_call_id,
                        "name": tr.name,
                        "content": json.dumps(tool_output, ensure_ascii=True),
                    }
                )
                continue

            if msg.role == pb2.Message.Role.ASSISTANT and msg.content:
                payload.append({"role": "assistant", "content": msg.content})

        return payload

    @staticmethod
    def _build_tools(tools: list[pb2.ToolDefinition]) -> list[dict[str, Any]]:
        payload: list[dict[str, Any]] = []
        for tool in tools:
            schema = OpenRouterClient._struct_to_dict(tool.parameters) if tool.parameters else {}
            payload.append(
                {
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": OpenRouterClient._normalize_schema(schema),
                    },
                }
            )

        return payload

    @staticmethod
    def _normalize_schema(schema: Any) -> Any:
        if isinstance(schema, dict):
            normalized: dict[str, Any] = {}
            for key, value in schema.items():
                if key == "type":
                    normalized[key] = OpenRouterClient._normalize_type(value)
                else:
                    normalized[key] = OpenRouterClient._normalize_schema(value)
            return normalized

        if isinstance(schema, list):
            return [OpenRouterClient._normalize_schema(item) for item in schema]

        return schema

    @staticmethod
    def _normalize_type(value: Any) -> Any:
        if isinstance(value, list):
            for candidate in value:
                if isinstance(candidate, str) and candidate.lower() != "null":
                    return candidate
            return "string"

        return value

    @staticmethod
    def _struct_to_dict(struct: struct_pb2.Struct) -> dict[str, Any] | None:
        if not struct or not struct.fields:
            return None
        return json_format.MessageToDict(struct, preserving_proto_field_name=True)
