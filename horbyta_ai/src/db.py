from __future__ import annotations

import os, asyncpg, structlog

logger = structlog.get_logger(__name__)

_pool: asyncpg.Pool | None = None

async def init_pool() -> None:
    global _pool
    
    dsn = os.environ.get("DATABASE_URL")
    if not dsn:
        raise RuntimeError("DATABASE_URL environment variable is not set")
    
    _pool = await asyncpg.create_pool(
        dsn=dsn,
        min_size=2,
        max_size=10,
        command_timeout=30,
        statement_cache_size=20
    )
    logger.info("db_pool_initialized")

async def close_pool() -> None:
    global _pool

    if _pool is not None:
        await _pool.close()
        _pool = None
        logger.info("db_pool_closed")

async def get_pool() -> asyncpg.Pool:
    if _pool is None:
        raise RuntimeError("Database pool has not been initialized")
    return _pool