#!/bin/bash
# Python server shutdown script
set -e

cd "$(dirname "$0")"

PIDFILE="uvicorn.pid"

echo "🛑 Stopping Python server..."

if [ ! -f "$PIDFILE" ]; then
    echo "⚠️  No server running (PID file not found)"
    exit 0
fi

PID=$(cat "$PIDFILE")

if ps -p "$PID" > /dev/null 2>&1; then
    echo "Stopping process $PID..."
    kill "$PID"
    
    # Wait for the process to terminate
    for i in {1..10}; do
        if ! ps -p "$PID" > /dev/null 2>&1; then
            break
        fi
        sleep 0.5
    done
    
    # If the process hasn't stopped, force shutdown
    if ps -p "$PID" > /dev/null 2>&1; then
        echo "Forcing shutdown..."
        kill -9 "$PID"
    fi
    
    rm "$PIDFILE"
    echo "✅ Server stopped"
else
    echo "⚠️  Process $PID is no longer running"
    rm "$PIDFILE"
fi
