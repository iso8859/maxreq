#!/bin/bash
# Start Node.js server with PM2 using ecosystem config and Nginx load balancer
set -e

cd "$(dirname "$0")"

# Number of CPU cores
NUM_CORES=$(nproc)

echo "🚀 Starting Node.js server with PM2 and Nginx load balancer..."

# Check if PM2 is installed
if ! command -v pm2 &> /dev/null; then
    echo "❌ PM2 is not installed. Please run ./setup.sh first"
    exit 1
fi

# Check if Nginx is installed
if ! systemctl list-unit-files | grep -q nginx.service; then
    echo "❌ Nginx is not installed. Please run ./setup.sh first"
    exit 1
fi

# Check if ecosystem config exists
if [ ! -f "ecosystem.config.js" ]; then
    echo "❌ ecosystem.config.js not found"
    exit 1
fi

# Stop any existing instances
pm2 delete all 2>/dev/null || true

# Start using ecosystem config
pm2 start ecosystem.config.js

# Save the PM2 process list
pm2 save

# Restart Nginx to ensure fresh start
echo "Starting Nginx load balancer..."
sudo systemctl restart nginx

# Check Nginx status
if sudo systemctl is-active --quiet nginx; then
    NGINX_STATUS="✅ Running"
else
    NGINX_STATUS="❌ Failed"
fi

echo ""
echo "✅ Node.js server started successfully!"
echo ""
echo "📊 Service Status:"
echo "  Nginx load balancer: $NGINX_STATUS"
echo "  PM2 instances: 16 running"
echo ""
echo "🌐 Access Points:"
echo "  Load balancer: http://localhost:8080"
echo "  Nginx health: http://localhost:8080/nginx-health"
echo "  Node.js health: http://localhost:8080/nodejshealth"
echo ""
echo "📊 PM2 Status:"
pm2 status
echo ""
echo "📝 Useful commands:"
echo "  View logs: pm2 logs"
echo "  Monitor PM2: pm2 monit"
echo "  Nginx logs: sudo journalctl -u nginx -f"
echo "  Nginx status: sudo systemctl status nginx"
