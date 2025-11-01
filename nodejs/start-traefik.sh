#!/bin/bash
# Start Traefik reverse proxy for Node.js API
set -e

cd "$(dirname "$0")"

echo "🚀 Starting Traefik reverse proxy..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null 2>&1; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Stop any existing Traefik container
docker-compose -f docker-compose.traefik.yml down 2>/dev/null || true

# Start Traefik
if docker compose version &> /dev/null 2>&1; then
    docker compose -f docker-compose.traefik.yml up -d
else
    docker-compose -f docker-compose.traefik.yml up -d
fi

echo ""
echo "✅ Traefik started successfully!"
echo ""
echo "🌐 Access Points:"
echo "  API via Traefik: http://localhost:8081"
echo "  Traefik Dashboard: http://localhost:8082"
echo ""
echo "📊 Traefik is load balancing across 16 Node.js instances (ports 3001-3016)"
echo ""
echo "💡 To stop: ./stop-traefik.sh"
