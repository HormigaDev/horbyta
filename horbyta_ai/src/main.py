from __future__ import annotations

import asyncio, os, signal, sys, grpc, structlog

from .conversation_repo import ConversationRepository
from .db import close_pool, init_pool
from .logging_config import configure_logging
from .servicer import AiServiceServicer

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'generated'))
from generated import horbyta_service_pb2_grpc as pb2_grpc

logger = structlog.get_logger(__name__)

_EVICTION_INTERVAL: int = int(os.environ.get("EVICTION_INTERVAL_SECONDS", "60"))

async def _run_eviction(repo: ConversationRepository) -> None:
    while True:
        await asyncio.sleep(_EVICTION_INTERVAL)
        try:
            await repo.purge_expired()
        except Exception:
            logger.exception("eviction_task_error")

async def serve() -> None:
    port = int(os.environ["GRPC_PORT"])

    await init_pool()

    repo = ConversationRepository()
    eviction_task = asyncio.create_task(_run_eviction(repo))

    server = grpc.aio.server()
    pb2_grpc.add_AiServiceServicer_to_server(AiServiceServicer(repo), server)

    listen_addr = f"0.0.0.0:{port}"
    server.add_insecure_port(listen_addr)
    await server.start()
    logger.info("grpc_server_started", address=listen_addr)

    stop_event = asyncio.Event()

    def _handle_signal(sig: int) -> None:
        logger.info("shutdown_signal_received", signal=sig)
        stop_event.set()

    loop = asyncio.get_running_loop()
    for sig in (signal.SIGTERM, signal.SIGINT):
        loop.add_signal_handler(sig, _handle_signal, sig)

    await stop_event.wait()

    logger.info("graceful_shuwdown_initiated")
    eviction_task.cancel()

    await server.stop(grace=10)
    await close_pool()
    logger.info("server_stopped")

def main() -> None:
    configure_logging()
    asyncio.run(serve())

if __name__ == "__main__":
    main()
