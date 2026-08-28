#!/usr/bin/env bash
#
# serve.sh — Start the E-Lupane application.
#
# Usage:
#   ./scripts/serve.sh           # dev mode  (hot-reload, separate processes)
#   ./scripts/serve.sh --prod    # prod mode (gateway supervises all plugins)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONTEND_DIR="$PROJECT_ROOT/plugins/frontend"
PORT="${PORT:-8090}"
PROD_MODE=false

for arg in "$@"; do
    case "$arg" in
        --prod) PROD_MODE=true ;;
        -h|--help)
            echo "Usage: $0 [--prod]"
            echo "  --prod  Production mode: gateway launches all plugins"
            echo "          (including the frontend) as child processes."
            exit 0
            ;;
    esac
done

export PATH="$HOME/.cargo/bin:$PATH"

BACKEND_LOG="/tmp/elupane_backend.log"
FRONTEND_LOG="/tmp/elupane_frontend.log"
FRONTEND_MANIFEST="$PROJECT_ROOT/plugins/frontend/plugin.toml"
FRONTEND_BIN="$FRONTEND_DIR/target/debug/frontend"

log() { echo "[serve.sh] $*"; }
check_cmd() { command -v "$1" >/dev/null 2>&1 || { log "ERROR: '$1' not found."; exit 1; }; }
check_cmd cargo

# =========================================================================
# PRODUCTION MODE — gateway launches everything
# =========================================================================
if [ "$PROD_MODE" = true ]; then
    log "Production mode: gateway will launch all plugins (including frontend)."

    if [ ! -f "$FRONTEND_MANIFEST" ]; then
        log "ERROR: $FRONTEND_MANIFEST not found."
        exit 1
    fi

    log "Building frontend binary..."
    cd "$FRONTEND_DIR"
    cargo build --features server
    if [ ! -f "$FRONTEND_BIN" ]; then
        log "ERROR: Frontend binary not found at $FRONTEND_BIN"
        exit 1
    fi
    log "Frontend binary ready: $FRONTEND_BIN"

    log "Starting gateway (supervisor)..."
    cd "$PROJECT_ROOT"
    nohup cargo run -p gateway > "$BACKEND_LOG" 2>&1 &
    GATEWAY_PID=$!
    echo "$GATEWAY_PID" > /tmp/elupane_gateway.pid
    log "Gateway PID: $GATEWAY_PID (logs: $BACKEND_LOG)"

    log ""
    log "=============================================="
    log "  E-Lupane (PRODUCTION) starting!"
    log "=============================================="
    log ""
    log "  Gateway supervisor -> cargo run -p gateway"
    log "    Launches all plugins:"
    log "    * Backend plugins (Assignments, Course Management, etc.)"
    log "    * Dioxus frontend (Axum server on port $PORT)"
    log ""
    log "  Browser: http://localhost:$PORT"
    log "  Stop:    kill \$(cat /tmp/elupane_gateway.pid)"
    log "=============================================="
    exit 0
fi

# =========================================================================
# DEV MODE — separate processes with hot-reload
# =========================================================================
log "Development mode: backend gateway + dx serve (hot-reload)."

# Temporarily disable the frontend plugin manifest to avoid port conflict
# with dx serve (which also binds port 8090).
if [ -f "$FRONTEND_MANIFEST" ]; then
    mv "$FRONTEND_MANIFEST" "${FRONTEND_MANIFEST}.disabled"
    log "Temporarily disabled frontend plugin manifest for dev mode."
    RESTORE_MANIFEST=true
else
    RESTORE_MANIFEST=false
fi

cleanup() {
    if [ "$RESTORE_MANIFEST" = true ] && [ -f "${FRONTEND_MANIFEST}.disabled" ]; then
        mv "${FRONTEND_MANIFEST}.disabled" "$FRONTEND_MANIFEST"
        log "Restored frontend plugin manifest."
    fi
}
trap cleanup EXIT

check_cmd dx

log "Starting backend engine (gateway)..."
cd "$PROJECT_ROOT"
nohup cargo run -p gateway > "$BACKEND_LOG" 2>&1 &
BACKEND_PID=$!
echo "$BACKEND_PID" > /tmp/elupane_backend.pid
log "Backend engine PID: $BACKEND_PID (logs: $BACKEND_LOG)"

sleep 2

log "Starting Dioxus frontend (dx serve --port $PORT)..."
cd "$FRONTEND_DIR"
nohup dx serve --port "$PORT" > "$FRONTEND_LOG" 2>&1 &
FRONTEND_PID=$!
echo "$FRONTEND_PID" > /tmp/elupane_frontend.pid
log "Frontend PID: $FRONTEND_PID (logs: $FRONTEND_LOG)"

log ""
log "=============================================="
log "  E-Lupane (DEV) starting!"
log "=============================================="
log ""
log "  Backend  -> cargo run -p gateway"
log "    PID:  $BACKEND_PID    Log: $BACKEND_LOG"
log ""
log "  Frontend -> dx serve --port $PORT"
log "    PID:  $FRONTEND_PID    Log: $FRONTEND_LOG"
log ""
log "  Browser: http://localhost:$PORT"
log "  Stop:    ./scripts/stop.sh"
log "=============================================="
