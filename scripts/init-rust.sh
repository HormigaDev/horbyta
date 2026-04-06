#!/usr/bin/env bash

set -euo pipefail

readonly MAX_ATTEMPTS=30
readonly RETRY_INTERVAL=2
readonly CHECK_ONLY="${1:-}"

if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "FATAL: DATABASE_URL is not set." >&2
    exit 1
fi

echo "Waiting for PostgreSQL..."

attempt=0
until pg_isready --dbname="$DATABASE_URL" --quiet; do
    attempt=$(( attempt + 1 ))
    if (( attempt >= MAX_ATTEMPTS )); then
        echo "FATAL: PostgreSQL did not beco ready after $(( MAX_ATTEMPTS * RETRY_INTERVAL ))s. Aborting." >&2
        exit 1
    fi
    echo " PostgreSQL not ready - attempt ${attempt}/${MAX_ATTEMPTS}. Retrying in ${RETRY_INTERVAL}s..."
    sleep "$RETRY_INTERVAL"
done

echo "PostgreSQL is ready."

if [[ "$CHECK_ONLY" == "check-only" ]]; then
    echo "check-only mode: skipping exec."
    exit 0
fi

echo "Starting server..."
exec ./server