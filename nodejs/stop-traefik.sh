#!/bin/bash
# Stop Traefik reverse proxy
set -e

cd "$(dirname "$0")"

echo "🛑 Stopping Traefik reverse proxy..."

# Stop Traefik container
if docker compose version &> /dev/null 2>&1; then
    docker compose -f docker-compose.traefik.yml down
else
    docker-compose -f docker-compose.traefik.yml down
fi

echo "✅ Traefik stopped"
