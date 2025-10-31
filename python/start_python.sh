#!/bin/bash
# Python server startup script with as many workers as CPU cores
set -e

cd "$(dirname "$0")"

# Number of CPU cores
NUM_CORES=$(nproc)

# Startup port
PORT="${1:-8000}"

# PID file
PIDFILE="uvicorn.pid"

echo "🚀 Starting Python server on port $PORT with $NUM_CORES workers..."

# Check if server is already running
if [ -f "$PIDFILE" ]; then
    PID=$(cat "$PIDFILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo "⚠️  Server is already running (PID: $PID)"
        exit 1
    else
        rm "$PIDFILE"
    fi
fi

# Activate virtual environment
if [ -d ".venv" ]; then
    source .venv/bin/activate
else
    echo "❌ Virtual environment not found. Run ./install_python_uv.sh first"
    exit 1
fi

# Start uvicorn with number of workers = number of cores
nohup uvicorn main:app --host 0.0.0.0 --port "$PORT" --workers "$NUM_CORES" > uvicorn.log 2>&1 &

# Save the PID
echo $! > "$PIDFILE"

echo "✅ Server started on http://0.0.0.0:$PORT with $NUM_CORES workers"
echo "📝 Logs: uvicorn.log"
echo "🔢 PID: $(cat $PIDFILE)"
