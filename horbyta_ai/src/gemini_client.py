from __future__ import annotations

import asyncio
import enum
import os
from dataclasses import dataclass
from typing import Any, AsyncIterator, cast

import structlog
from google import genai
from google.genai import types

logger = structlog.get_logger(__name__)

_MODEL = "gemini-2.0-flash"
_MAX_RETRIES = 3
_RETRY_BACKOFF_BASE = 1.5
_STREAM_END = object()


class ChunkKind(enum.Enum):
    TEXT_DELTA = "text_delta"
    TOOL_CALL = "tool_call"
    DONE = "done"


@dataclass
class GeminiChunk:
    kind: ChunkKind
    data: Any  # str for TEXT_DELTA | dict for TOOL_CALL | None for DONE


class GeminiClient:
    def __init__(self) -> None:
        api_key = os.environ.get("GEMINI_API_KEY")
        if not api_key:
            raise RuntimeError("GEMINI_API_KEY environment variable is not set")
        self._client = genai.Client(api_key=api_key)

    async def stream_turn(
        self,
        messages: list[dict],
        tools: list[dict],
    ) -> AsyncIterator[GeminiChunk]:
        """
        Streams a single conversational turn.
        `messages` is the full history loaded from the DB.
        """
        contents = self._to_gemini_contents(messages)
        gemini_tools = self._build_tools(tools) if tools else None

        for attempt in range(1, _MAX_RETRIES + 1):
            try:
                async for chunk in self._do_stream(contents, gemini_tools):
                    yield chunk
                return
            except Exception as exc:
                if attempt == _MAX_RETRIES:
                    logger.error("gemini_stream_failed", error=str(exc), attempts=attempt)
                    raise
                wait = _RETRY_BACKOFF_BASE ** attempt
                logger.warning(
                    "gemini_stream_retrying",
                    error=str(exc),
                    attempt=attempt,
                    wait_seconds=wait,
                )
                await asyncio.sleep(wait)

    async def _do_stream(
        self,
        contents: list[types.Content],
        tools: list[types.Tool] | None,
    ) -> AsyncIterator[GeminiChunk]:
        kwargs: dict = {"model": _MODEL, "contents": contents}
        if tools:
            kwargs["tools"] = tools

        # SDK returns a sync iterator, so pull items via thread hops to keep async flow non-blocking.
        stream = await asyncio.to_thread(self._client.models.generate_content_stream, **kwargs)

        while True:
            raw_response = await asyncio.to_thread(next, stream, _STREAM_END)
            if raw_response is _STREAM_END:
                break

            response = cast(types.GenerateContentResponse, raw_response)
            candidates = response.candidates or []
            if not candidates:
                continue

            content = candidates[0].content
            if content is None or content.parts is None:
                continue

            for part in content.parts:
                if part.text:
                    yield GeminiChunk(kind=ChunkKind.TEXT_DELTA, data=part.text)

                elif part.function_call:
                    fc = part.function_call
                    yield GeminiChunk(
                        kind=ChunkKind.TOOL_CALL,
                        data={
                            "id": fc.id or fc.name,
                            "name": fc.name,
                            "arguments": fc.args if isinstance(fc.args, dict) else {},
                        },
                    )

        yield GeminiChunk(kind=ChunkKind.DONE, data=None)

    @staticmethod
    def _to_gemini_contents(messages: list[dict]) -> list[types.Content]:
        contents: list[types.Content] = []

        for msg in messages:
            role = msg["role"]

            if role in ("user", "model") and "content" in msg:
                contents.append(
                    types.Content(
                        role=role,
                        parts=[types.Part(text=msg["content"])],
                    )
                )

            elif role == "model" and "tool_call" in msg:
                tc = msg["tool_call"]
                contents.append(
                    types.Content(
                        role="model",
                        parts=[
                            types.Part(
                                function_call=types.FunctionCall(
                                    id=tc["id"],
                                    name=tc["name"],
                                    args=tc["arguments"],
                                )
                            )
                        ],
                    )
                )

            elif role == "tool":
                contents.append(
                    types.Content(
                        role="tool",
                        parts=[
                            types.Part(
                                function_response=types.FunctionResponse(
                                    id=msg["tool_call_id"],
                                    name=msg["name"],
                                    response=msg["content"],
                                )
                            )
                        ],
                    )
                )

        return contents

    @staticmethod
    def _build_tools(tools: list[dict]) -> list[types.Tool]:
        declarations = [
            types.FunctionDeclaration(
                name=t["name"],
                description=t["description"],
                parameters=t.get("parameters"),
            )
            for t in tools
        ]
        tool_ctor = cast(Any, types.Tool)
        return [tool_ctor(function_declarations=declarations)]