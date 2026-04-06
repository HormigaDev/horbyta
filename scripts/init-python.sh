#!/usr/bin/env bash

set -euo pipefail

missing=()

[[ -z "${GEMINI_API_KEY:-}" ]] && missing+=("GEMINI_API_KEY")
[[ -z "${GRPC_PORT:-}" ]] && missing+=("GRPC_PORT")

if (( ${#missing[@]} > 0 )); then
    echo "FATAL: missing required environment variables: ${missing[*]}" >&2
    exit 1
fi

echo "Environment validated. Starting AI service on port ${GRPC_PORT}..."

exec /opt/venv/bin/python -m src.main