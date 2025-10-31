#!/bin/bash
# Stop Nginx + PHP-FPM server
set -e

cd "$(dirname "$0")"

echo "🛑 Stopping Nginx + PHP-FPM server..."

# Stop services
echo "Stopping Nginx..."
sudo systemctl stop nginx

echo "Stopping PHP-FPM..."
sudo systemctl stop php8.2-fpm

# Remove site configuration link
if [ -L "/etc/nginx/sites-enabled/php-api" ]; then
    echo "Removing Nginx site configuration..."
    sudo rm /etc/nginx/sites-enabled/php-api
fi

echo "✅ Server stopped successfully"
echo ""
echo "💡 To completely remove the configuration:"
echo "   sudo rm /etc/nginx/sites-available/php-api"
