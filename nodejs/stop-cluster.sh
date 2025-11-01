#!/bin/bash
# Stop Node.js cluster server
set -e

cd "$(dirname "$0")"

echo "🛑 Stopping Node.js cluster server..."

if [ -f server.pid ]; then
    PID=$(cat server.pid)
    if ps -p $PID > /dev/null; then
        kill $PID
        echo "✅ Server stopped (PID: $PID)"
        rm server.pid
    else
        echo "⚠️  Server not running (stale PID file removed)"
        rm server.pid
    fi
else
    # Try to kill by process name
    pkill -f "node index.js" 2>/dev/null && echo "✅ Server stopped" || echo "⚠️  Server not running"
fi
