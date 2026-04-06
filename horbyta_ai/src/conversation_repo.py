from __future__ import annotations

import json, os, structlog

from typing import Any
from .db import get_pool

logger = structlog.get_logger(__name__)

_TTL_SECONDS: int = int(os.environ.get("SESSION_TTL_SECONDS", "1800"))

class ConversationRepository:
    """Persistence layer for chat conversation history."""

    async def load_messages(self, user_id: str, session_id: str) -> list[dict]:
        pool = await get_pool()
        async with pool.acquire() as conn:
            row = await conn.fetchrow(
                """
                SELECT messages
                FROM conversation_history
                WHERE user_id = $1::uuid AND session_id = $2::uuid
                """,
                user_id,
                session_id
            )
        if row is None:
            return []
        return json.loads(row["messages"])
    
    async def append_user_message(
        self,
        user_id: str,
        session_id: str,
        content: str
    ) -> None:
        await self._append(user_id, session_id, {"role": "user", "content": content})

    async def append_model_text(
        self,
        user_id: str,
        session_id: str,
        content: str,
    ) -> None:
        await self._append(user_id, session_id, {"role": "model", "content": content})

    async def append_model_tool_call(
        self,
        user_id: str,
        session_id: str,
        tool_call_id: str,
        name: str,
        arguments: dict[str, Any],
    ) -> None:
        await self._append(
            user_id,
            session_id,
            {
                "role": "model",
                "tool_call": {
                    "id": tool_call_id,
                    "name": name,
                    "arguments": arguments
                }
            }
        )

    async def append_tool_result(
        self,
        user_id: str,
        session_id: str,
        tool_call_id: str,
        name: str,
        result: Any,
        success: bool,
        error_message: str | None,
    ) -> None:
        content = result if success else {"error": error_message or "unknown error"}
        await self._append(
            user_id,
            session_id,
            {
                "role": "tool",
                "tool_call_id": tool_call_id,
                "name": name,
                "content": content,
            },
        )

    async def purge_expired(self) -> int:
        pool = await get_pool()
        async with pool.acquire() as conn:
            result = await conn.execute(
                """
                DELETE from conversation_history
                WHERE updated_at < now() - ($1 * interval '1 second')
                """,
                _TTL_SECONDS,
            )
        count = int(result.split()[-1])
        if count > 0:
            logger.info("conversations_purged", count=count, ttl_seconds=_TTL_SECONDS)
        return count
    
    async def _append(
        self,
        user_id: str,
        session_id: str,
        message: dict,
    ) -> None:
        pool = await get_pool()
        async with pool.acquire() as conn:
            await conn.execute(
                """
                INSERT INTO conversation_history (user_id, session_id, messages)
                VALUES ($1::uuid, $2::uuid, $3::jsonb)
                ON CONFLICT ON CONSTRAINT uq_conversation_user_session DO UPDATE
                SET messages = conversation_history.messages || EXCLUDED.messages
                """,
                user_id,
                session_id,
                json.dumps([message]),
            )
