#!/bin/bash
# Stop Node.js server managed by PM2 and Nginx
set -e

cd "$(dirname "$0")"

echo "🛑 Stopping Node.js server and Nginx load balancer..."

# Stop Nginx if installed
if systemctl list-unit-files | grep -q nginx.service; then
    echo "Stopping Nginx..."
    sudo systemctl stop nginx
    echo "✅ Nginx stopped"
fi

# Check if PM2 is installed
if ! command -v pm2 &> /dev/null; then
    echo "⚠️  PM2 is not installed"
    exit 0
fi

# Stop all PM2 processes
pm2 stop all

# Optionally delete all processes (uncomment if you want to remove them completely)
# pm2 delete all

# Save the PM2 process list (will be empty after stop)
pm2 save

echo "✅ Node.js server stopped"
echo ""
echo "📊 PM2 Status:"
pm2 status
echo ""
echo "💡 To completely remove all processes: pm2 delete all"
echo "💡 To restart: ./start.sh"
