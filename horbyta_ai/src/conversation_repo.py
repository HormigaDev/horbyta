from __future__ import annotations

import json
import os
import sys
import structlog

from typing import Any
from google.protobuf import json_format
from .db import get_pool

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'generated'))
from generated import horbyta_service_pb2 as pb2

logger = structlog.get_logger(__name__)

_TTL_SECONDS: int = int(os.environ.get("SESSION_TTL_SECONDS", "1800"))


class ConversationRepository:
    """
    Persistence layer for chat conversation history.
    Stores and retrieves protobuf Message[] with roles.
    """

    async def load_messages(self, user_id: str, session_id: str) -> list[pb2.Message]:
        """Load conversation history as protobuf Message list."""
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
        
        messages_json = json.loads(row["messages"])
        return [
            json_format.ParseDict(msg_dict, pb2.Message())
            for msg_dict in messages_json
        ]

    async def append_message(
        self,
        user_id: str,
        session_id: str,
        message: pb2.Message | dict,
    ) -> None:
        """
        Append a message to the conversation history.
        Accepts either a protobuf Message or a dict (for backward compatibility).
        """
        if isinstance(message, pb2.Message):
            msg_dict = json_format.MessageToDict(message)
        else:
            msg_dict = message
        
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
                json.dumps([msg_dict]),
            )

    async def purge_expired(self) -> int:
        """Delete conversations older than TTL."""
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
