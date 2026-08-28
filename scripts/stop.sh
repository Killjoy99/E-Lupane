#!/usr/bin/env bash
#
# stop.sh — Stop the E-Lupane backend engine and Dioxus frontend.
#
# Usage:  ./scripts/stop.sh
#
set -euo pipefail

log() { echo "[stop.sh] $*"; }

# --- Stop frontend ---
if [ -f /tmp/elupane_frontend.pid ]; then
    PID=$(cat /tmp/elupane_frontend.pid)
    if kill -0 "$PID" 2>/dev/null; then
        log "Stopping frontend (PID $PID)..."
        kill "$PID" 2>/dev/null || true
        sleep 1
        kill -9 "$PID" 2>/dev/null || true
    else
        log "Frontend (PID $PID) is not running."
    fi
    rm -f /tmp/elupane_frontend.pid
else
    log "No frontend PID file found."
fi

# Also kill any lingering dx serve processes
pkill -f "dx serve" 2>/dev/null || true

# --- Stop backend / gateway (handles both dev and prod modes) ---
# Production mode: gateway PID is in /tmp/elupane_gateway.pid
if [ -f /tmp/elupane_gateway.pid ]; then
    PID=$(cat /tmp/elupane_gateway.pid)
    if kill -0 "$PID" 2>/dev/null; then
        log "Stopping gateway supervisor (PID $PID)..."
        kill "$PID" 2>/dev/null || true
        sleep 2
        kill -9 "$PID" 2>/dev/null || true
        # Also kill any child plugin/frontend processes spawned by the gateway
        pkill -f "target/debug/gateway" 2>/dev/null || true
        pkill -f "frontend/target/debug/frontend" 2>/dev/null || true
    else
        log "Gateway (PID $PID) is not running."
    fi
    rm -f /tmp/elupane_gateway.pid
fi

# Development mode: backend is in /tmp/elupane_backend.pid
if [ -f /tmp/elupane_backend.pid ]; then
    PID=$(cat /tmp/elupane_backend.pid)
    if kill -0 "$PID" 2>/dev/null; then
        log "Stopping backend engine (PID $PID)..."
        kill "$PID" 2>/dev/null || true
        sleep 1
        kill -9 "$PID" 2>/dev/null || true
    else
        log "Backend engine (PID $PID) is not running."
    fi
    rm -f /tmp/elupane_backend.pid
fi

# Kill any lingering gateway/plugin/frontend processes
pkill -f "target/debug/gateway" 2>/dev/null || true
pkill -f "frontend/target/debug/frontend" 2>/dev/null || true

log "Done."
