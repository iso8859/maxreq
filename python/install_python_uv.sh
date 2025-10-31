#!/bin/bash
# Python and dependencies installation script with UV
set -e

cd "$(dirname "$0")"

echo "📦 Installing Python and UV..."

# Install Python 3 if necessary
if ! command -v python3 &> /dev/null; then
    echo "Installing Python 3..."
    sudo apt-get update
    sudo apt-get install -y python3 python3-pip python3-venv
fi

# Install UV (fast Python package manager)
if ! command -v uv &> /dev/null; then
    echo "Installing UV..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Create a virtual environment with UV
echo "Creating virtual environment..."
uv venv

# Install dependencies
echo "Installing dependencies..."
uv pip install -r requirements.txt

echo "✅ Installation completed!"
echo "To start the server: ./start_python.sh"
