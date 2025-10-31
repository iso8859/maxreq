#!/bin/bash
# PHP server setup script for Debian - Using Nginx + PHP-FPM
set -e

cd "$(dirname "$0")"

echo "📦 Installing Nginx, PHP-FPM and extensions..."

# Install Nginx, PHP-FPM and required extensions
echo "Installing Nginx and PHP 8.2..."
sudo apt-get update
sudo apt-get install -y nginx php8.2-fpm php8.2-sqlite3 php8.2-mbstring php8.2-xml

# Configure PHP-FPM for performance
echo "Configuring PHP-FPM for optimal performance..."

# Get number of cores
NUM_CORES=$(nproc)

# Backup original config
sudo cp /etc/php/8.2/fpm/pool.d/www.conf /etc/php/8.2/fpm/pool.d/www.conf.backup 2>/dev/null || true

# Configure PHP-FPM pool with dynamic workers based on cores
sudo tee /etc/php/8.2/fpm/pool.d/www.conf > /dev/null <<EOF
[www]
user = www-data
group = www-data
listen = /run/php/php8.2-fpm.sock
listen.owner = www-data
listen.group = www-data
listen.mode = 0660

pm = dynamic
pm.max_children = $((NUM_CORES * 4))
pm.start_servers = $NUM_CORES
pm.min_spare_servers = $NUM_CORES
pm.max_spare_servers = $((NUM_CORES * 2))
pm.max_requests = 500

; Performance tuning
pm.process_idle_timeout = 10s
request_terminate_timeout = 60s
EOF

echo "✅ Installation completed!"
echo "PHP-FPM configured with $NUM_CORES start servers and up to $((NUM_CORES * 4)) max children"
echo "To start the server: ./start.sh"
