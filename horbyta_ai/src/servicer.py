from __future__ import annotations

import asyncio, json, os, sys, threading, grpc, structlog

from typing import Any
from .conversation_repo import ConversationRepository
from .gemini_client import GeminiClient, ChunkKind

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'generated'))
from generated import horbyta_service_pb2 as pb2, horbyta_service_pb2_grpc as pb2_grpc # noqa: E402

logger = structlog.get_logger(__name__)

_TOOL_WAIT_TIMEOUT: float = float(os.environ.get("TOOL_WAIT_TIMEOUT_SECONDS", "30"))
_MAX_TOOL_ROUNDS: int = 10

class AiServiceServicer(pb2_grpc.AiServiceServicer):
    """gRPC servicer that orchestrates chat turns and tool-call roundtrips."""

    def __init__(self, repo: ConversationRepository) -> None:
        self._gemini = GeminiClient()
        self._repo = repo

        self._pending_events: dict[str, asyncio.Event] = {}
        self._pending_results: dict[str, dict] = {}
        self._pending_tool_names: dict[str, str] = {}
        self._pending_lock = threading.Lock()

    @staticmethod
    def _parse_json_field(value: str, field_name: str) -> Any:
        """Parse a JSON payload field and raise ValueError with a stable message."""
        if not value:
            return {}
        try:
            return json.loads(value)
        except json.JSONDecodeError as exc:
            raise ValueError(f"invalid JSON in '{field_name}'") from exc

    async def Chat(
        self,
        request: pb2.ChatRequest,
        context: grpc.aio.ServicerContext
    ):
        log = logger.bind(user_id=request.user_id, session_id=request.session_id)

        if not request.user_id or not request.session_id:
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, "user_id and session_id are required")
            return
        
        if not request.user_message.strip():
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, "user_messae cannot be empty")
            return

        tools: list[dict[str, Any]] = []
        try:
            tools = [
                {
                    "name": t.name,
                    "description": t.description,
                    "parameters": self._parse_json_field(t.parameters_json, "available_tools.parameters_json"),
                }
                for t in request.available_tools
            ]
        except ValueError as exc:
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, str(exc))
            return

        try:
            await self._repo.append_user_message(
                request.user_id,
                request.session_id,
                request.user_message
            )
        except Exception as exc:
            log.error("db_append_user_message_failed", error=str(exc))
            yield pb2.ChatChunk(error="Failed to persist message")
            return
        
        log.info("chat_started", tools=len(tools))

        accumulated_text: list[str] = []

        for round_num in range(1, _MAX_TOOL_ROUNDS + 1):
            try:
                messages = await self._repo.load_messages(
                    request.user_id,
                    request.session_id
                )
            except Exception as exc:
                log.error("db_load_failed", error=str(exc))
                yield pb2.ChatChunk(error="Failed to load conversation history")
                return
            
            tool_call_this_round: dict | None = None

            try:
                async for chunk in self._gemini.stream_turn(messages, tools):
                    if chunk.kind == ChunkKind.TEXT_DELTA:
                        accumulated_text.append(chunk.data)
                        yield pb2.ChatChunk(text_delta=chunk.data)
                    
                    elif chunk.kind == ChunkKind.TOOL_CALL:
                        tool_call_this_round = chunk.data

                        await self._repo.append_model_tool_call(
                            user_id=request.user_id,
                            session_id=request.session_id,
                            tool_call_id=chunk.data["id"],
                            name=chunk.data["name"],
                            arguments=chunk.data["arguments"],
                        )
                        yield pb2.ChatChunk(
                            tool_call=pb2.ToolCall(
                                id=chunk.data["id"],
                                name=chunk.data["name"],
                                arguments_json=json.dumps(chunk.data["arguments"]),
                            )
                        )
                        break
                    elif chunk.kind == ChunkKind.DONE:
                        break
            except Exception as exc:
                log.error("gemini_stream_error", error=str(exc), round=round_num)
                yield pb2.ChatChunk(error=f"Model error: {type(exc).__name__}")
                return
            
            if tool_call_this_round is None:
                if accumulated_text:
                    await self._repo.append_model_text(
                        request.user_id,
                        request.session_id,
                        "".join(accumulated_text),
                    )
                break

            result = await self._await_tool_result(
                session_id=request.session_id,
                tool_call_id=tool_call_this_round["id"],
                tool_name=tool_call_this_round["name"],
            )

            if result is None:
                log.warning("tool_result_timed_out", tool_call_id=tool_call_this_round["id"])
                yield pb2.ChatChunk(error="Tool execution timed out")
                return
            
            try:
                await self._repo.append_tool_result(
                    user_id=request.user_id,
                    session_id=request.session_id,
                    tool_call_id=result["tool_call_id"],
                    name=result["name"],
                    result=result["result"],
                    success=result["success"],
                    error_message=result.get("error_message"),
                )
            except Exception as exc:
                log.error("db_append_tool_result_failed", error=str(exc))
                yield pb2.ChatChunk(error="Failed to persist tool result")
                return
            
            accumulated_text = []
            log.debug("tool_round_complete", round=round_num)
        else:
            log.warning("max_tool_rounds_reached", max=_MAX_TOOL_ROUNDS)
            yield pb2.ChatChunk(error="exceeded maximum tool call rounds")
            return
        
        log.info("chat_complete")
        yield pb2.ChatChunk(done="")

    async def SubmitToolResult(
        self,
        request: pb2.ToolResultRequest,
        context: grpc.aio.ServicerContext
    ) -> pb2.ToolResultResponse:
        log = logger.bind(
            session_id=request.session_id,
            tool_call_id=request.tool_call_id,
        )

        if not request.session_id or not request.tool_call_id:
            await context.abort(
                grpc.StatusCode.INVALID_ARGUMENT,
                "session_id and tool_call_id are required",
            )
            return pb2.ToolResultResponse()

        try:
            result_data = self._parse_json_field(request.result_json, "result_json")
        except ValueError as exc:
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, str(exc))
            return pb2.ToolResultResponse()

        key = self._pending_key(request.session_id, request.tool_call_id)

        missing_waiter = False
        with self._pending_lock:
            event = self._pending_events.get(key)
            tool_name = self._pending_tool_names.get(key, request.tool_call_id)
            if event is None:
                missing_waiter = True
            else:
                self._pending_results[key] = {
                    "tool_call_id": request.tool_call_id,
                    "name": tool_name,
                    "result": result_data,
                    "success": request.success,
                    "error_message": request.error_message or None,
                }
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
        tool_name: str,
    ) -> dict | None:
        """Wait for a tool result correlated by session and tool-call id."""
        key = self._pending_key(session_id, tool_call_id)
        event = asyncio.Event()

        with self._pending_lock:
            self._pending_events[key] = event
            self._pending_tool_names[key] = tool_name

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