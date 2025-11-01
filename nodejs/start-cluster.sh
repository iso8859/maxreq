#!/bin/bash
# Start Node.js server using cluster mode
set -e

cd "$(dirname "$0")"

PORT="${1:-8080}"
NUM_WORKERS="${2:-$(nproc)}"

echo "🚀 Starting Node.js cluster server..."
echo "📍 Port: $PORT"
echo "⚙️  Workers: $NUM_WORKERS"

# Stop any existing Node.js process on this port
pkill -f "node index.js" 2>/dev/null || true

# Set environment variables and start
export PORT=$PORT
export WEB_CONCURRENCY=$NUM_WORKERS

# Start in background with nohup
nohup node index.js > server.log 2>&1 &
SERVER_PID=$!

# Save PID to file
echo $SERVER_PID > server.pid

# Wait a moment to check if it started successfully
sleep 2

if ps -p $SERVER_PID > /dev/null; then
    echo ""
    echo "✅ Node.js cluster server started successfully!"
    echo "📊 Master PID: $SERVER_PID"
    echo "🌐 Listening on: http://localhost:$PORT"
    echo "👷 Workers: $NUM_WORKERS"
    echo ""
    echo "📝 View logs: tail -f server.log"
    echo "🛑 Stop server: ./stop-cluster.sh"
else
    echo "❌ Failed to start server. Check server.log for errors."
    exit 1
fi
