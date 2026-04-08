from __future__ import annotations

import asyncio
import os
import sys
import threading
import grpc
import structlog

from typing import Any
from google.protobuf import json_format, struct_pb2
from .conversation_repo import ConversationRepository
from .openrouter_client import OpenRouterClient, ChunkKind, OpenRouterRequestError

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'generated'))
from generated import horbyta_service_pb2 as pb2, horbyta_service_pb2_grpc as pb2_grpc

logger = structlog.get_logger(__name__)

_TOOL_WAIT_TIMEOUT: float = float(os.environ.get("TOOL_WAIT_TIMEOUT_SECONDS", "30"))
_MAX_TOOL_ROUNDS: int = 10
_FALLBACK_ERROR_MESSAGE = "Ocurrio un error temporal procesando la respuesta. Intenta nuevamente."


class AiServiceServicer(pb2_grpc.AiServiceServicer):
    """
    gRPC servicer that orchestrates multi-turn conversations with tool use.
    Expects context metadata headers: x-user-id, x-session-id.
    """

    def __init__(self, repo: ConversationRepository) -> None:
        self._model_client = OpenRouterClient()
        self._repo = repo

        self._pending_events: dict[str, asyncio.Event] = {}
        self._pending_results: dict[str, pb2.ToolResult] = {}
        self._pending_tool_names: dict[str, str] = {}
        self._pending_lock = threading.Lock()

    @staticmethod
    def _extract_context(context: grpc.aio.ServicerContext) -> tuple[str, str, str, str]:
        """
        Extract user_id, session_id, model and api_key from gRPC context metadata.
        Returns (user_id, session_id, model, api_key) or raises if missing.
        """
        metadata = context.invocation_metadata() or []
        user_id = ""
        session_id = ""
        model = ""
        api_key = ""
        for key, value in metadata:
            if key == "x-user-id":
                user_id = value
            elif key == "x-session-id":
                session_id = value
            elif key == "x-model":
                model = value
            elif key == "x-api-key":
                api_key = value
        return user_id, session_id, model, api_key

    @staticmethod
    def _derive_title(messages: list[pb2.Message]) -> str:
        """Derive a stable title from the first user message, capped to 255 chars."""
        for message in messages:
            if message.role != pb2.Message.Role.USER or not message.content:
                continue

            words = message.content.split()
            if not words:
                break

            return " ".join(words[:3])[:255]

        for message in messages:
            if message.content:
                words = message.content.split()
                if words:
                    return " ".join(words[:3])[:255]

        return "Nuevo chat"

    async def Chat(
        self,
        request: pb2.ChatRequest,
        context: grpc.aio.ServicerContext
    ):
        user_id, session_id, model, api_key = self._extract_context(context)
        log = logger.bind(user_id=user_id, session_id=session_id, model=model)

        if not user_id or not session_id or not model or not api_key:
            await context.abort(
                grpc.StatusCode.INVALID_ARGUMENT,
                "Missing x-user-id, x-session-id, x-model or x-api-key in metadata",
            )
            return

        if not request.messages:
            await context.abort(
                grpc.StatusCode.INVALID_ARGUMENT,
                "messages cannot be empty",
            )
            return

        conversation_title = self._derive_title(list(request.messages))
        await self._repo.ensure_conversation(user_id, session_id, conversation_title)

        log.info("chat_started", message_count=len(request.messages), tool_count=len(request.available_tools))

        accumulated_text: list[str] = []
        tool_call_this_round: pb2.ToolCall | None = None
        assistant_tool_call_message: pb2.Message | None = None
        messages = list(request.messages)

        for round_num in range(1, _MAX_TOOL_ROUNDS + 1):
            try:
                async for chunk in self._model_client.stream_turn(
                    messages,
                    list(request.available_tools),
                    model=model,
                    api_key=api_key,
                ):
                    if chunk.kind == ChunkKind.TEXT_DELTA:
                        accumulated_text.append(chunk.data)
                        yield pb2.ChatEvent(text_delta=pb2.TextDelta(text=chunk.data))

                    elif chunk.kind == ChunkKind.TOOL_CALL:
                        tool_call_data = chunk.data
                        tool_call_this_round = pb2.ToolCall(
                            id=tool_call_data["id"],
                            name=tool_call_data["name"],
                            arguments=json_format.ParseDict(
                                tool_call_data.get("arguments", {}),
                                struct_pb2.Struct()
                            ),
                        )

                        try:
                            model_tool_call_message = pb2.Message(
                                role=pb2.Message.Role.ASSISTANT,
                                tool_call=tool_call_this_round,
                            )
                            assistant_tool_call_message = model_tool_call_message

                            with self._pending_lock:
                                self._pending_tool_names[
                                    self._pending_key(session_id, tool_call_this_round.id)
                                ] = tool_call_this_round.name

                            await self._repo.append_message(
                                user_id,
                                session_id,
                                model_tool_call_message,
                                title=conversation_title,
                            )
                        except Exception as exc:
                            log.error("db_append_tool_call_failed", error=str(exc))
                            yield pb2.ChatEvent(
                                error=pb2.Error(code="DB_ERROR", message="Failed to persist tool call")
                            )
                            return

                        yield pb2.ChatEvent(tool_call=tool_call_this_round)
                        break

                    elif chunk.kind == ChunkKind.DONE:
                        break

            except Exception as exc:
                if isinstance(exc, OpenRouterRequestError):
                    fallback_message = exc.user_message
                    log.error(
                        "llm_stream_error",
                        error_code=exc.code,
                        error=exc.user_message,
                        round=round_num,
                        details=exc.details,
                    )
                else:
                    fallback_message = _FALLBACK_ERROR_MESSAGE
                    log.error("llm_stream_error", error=str(exc), round=round_num)

                await self._emit_fallback_message(
                    user_id,
                    session_id,
                    conversation_title,
                    fallback_message,
                )
                yield pb2.ChatEvent(text_delta=pb2.TextDelta(text=fallback_message))
                yield pb2.ChatEvent(done=pb2.Done())
                return

            if tool_call_this_round is None:
                if accumulated_text:
                    text_content = "".join(accumulated_text)
                    try:
                        assistant_message = pb2.Message(
                            role=pb2.Message.Role.ASSISTANT,
                            content=text_content,
                        )
                        await self._repo.append_message(
                            user_id,
                            session_id,
                            assistant_message,
                            title=conversation_title,
                        )
                    except Exception as exc:
                        log.error("db_append_text_failed", error=str(exc))
                        yield pb2.ChatEvent(
                            error=pb2.Error(code="DB_ERROR", message="Failed to persist response")
                        )
                        return
                break

            tool_result = await self._await_tool_result(session_id, tool_call_this_round.id)

            if tool_result is None:
                log.warning("tool_result_timed_out", tool_call_id=tool_call_this_round.id)

                await self._emit_fallback_message(
                    user_id,
                    session_id,
                    conversation_title,
                    _FALLBACK_ERROR_MESSAGE,
                )
                yield pb2.ChatEvent(text_delta=pb2.TextDelta(text=_FALLBACK_ERROR_MESSAGE))
                yield pb2.ChatEvent(done=pb2.Done())
                return

            try:
                system_message = pb2.Message(
                    role=pb2.Message.Role.SYSTEM,
                    tool_result=tool_result,
                )
                await self._repo.append_message(
                    user_id,
                    session_id,
                    system_message,
                    title=conversation_title,
                )
            except Exception as exc:
                log.error("db_append_tool_result_failed", error=str(exc))
                yield pb2.ChatEvent(
                    error=pb2.Error(code="DB_ERROR", message="Failed to persist tool result")
                )
                return

            # Keep the exact assistant tool-call message in the round context before
            # appending the tool result. Anthropic/OpenRouter validates this order.
            if assistant_tool_call_message is not None:
                messages.append(assistant_tool_call_message)
            messages.append(system_message)
            accumulated_text = []
            tool_call_this_round = None
            assistant_tool_call_message = None
            log.debug("tool_round_complete", round=round_num)
        else:
            log.warning("max_tool_rounds_reached", max=_MAX_TOOL_ROUNDS)
            yield pb2.ChatEvent(
                error=pb2.Error(code="MAX_ROUNDS", message="Exceeded maximum tool call rounds")
            )
            return

        log.info("chat_complete")
        yield pb2.ChatEvent(done=pb2.Done())

    async def SubmitToolResult(
        self,
        request: pb2.ToolResultRequest,
        context: grpc.aio.ServicerContext
    ) -> pb2.ToolResultResponse:
        _, session_id, _, _ = self._extract_context(context)
        log = logger.bind(session_id=session_id, tool_call_id=request.tool_call_id)

        if not session_id or not request.tool_call_id:
            await context.abort(
                grpc.StatusCode.INVALID_ARGUMENT,
                "Missing x-session-id in metadata or tool_call_id in request",
            )
            return pb2.ToolResultResponse()

        key = self._pending_key(session_id, request.tool_call_id)

        with self._pending_lock:
            tool_name = self._pending_tool_names.get(key, "unknown_tool")

        tool_result = pb2.ToolResult(
            tool_call_id=request.tool_call_id,
            name=tool_name,
            success=request.success,
            error_message=request.error_message or "",
        )

        if request.result:
            tool_result.result.CopyFrom(request.result)

        missing_waiter = False
        with self._pending_lock:
            event = self._pending_events.get(key)
            if event is None:
                missing_waiter = True
            else:
                self._pending_results[key] = tool_result
                event.set()

        if missing_waiter:
            log.warning("tool_result_no_pending_waiter")
            await context.abort(
                grpc.StatusCode.NOT_FOUND,
                f"No pending chat waiting for tool_call_id '{request.tool_call_id}'",
            )
            return pb2.ToolResultResponse()

        log.info("tool_result_delivered", success=request.success)
        return pb2.ToolResultResponse()

    @staticmethod
    def _pending_key(session_id: str, tool_call_id: str) -> str:
        return f"{session_id}:{tool_call_id}"

    async def _await_tool_result(
        self,
        session_id: str,
        tool_call_id: str,
    ) -> pb2.ToolResult | None:
        """Wait for a tool result correlated by session and tool-call id."""
        key = self._pending_key(session_id, tool_call_id)
        event = asyncio.Event()

        with self._pending_lock:
            self._pending_events[key] = event

        try:
            await asyncio.wait_for(event.wait(), timeout=_TOOL_WAIT_TIMEOUT)
            with self._pending_lock:
                return self._pending_results.pop(key, None)
        except asyncio.TimeoutError:
            return None
        finally:
            with self._pending_lock:
                self._pending_events.pop(key, None)
                self._pending_results.pop(key, None)
                self._pending_tool_names.pop(key, None)

    async def _emit_fallback_message(
        self,
        user_id: str,
        session_id: str,
        conversation_title: str,
        content: str,
    ) -> None:
        """Persist a generic model reply when downstream model/tool chaining fails."""
        try:
            fallback_message = pb2.Message(
                role=pb2.Message.Role.ASSISTANT,
                content=content,
            )
            await self._repo.append_message(
                user_id,
                session_id,
                fallback_message,
                title=conversation_title,
            )
        except Exception as exc:
            logger.error("db_append_fallback_failed", error=str(exc), session_id=session_id)