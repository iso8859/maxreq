#!/bin/bash
# Start Nginx + PHP-FPM server
set -e

cd "$(dirname "$0")"

# Port
PORT="${1:-8080}"

# Get absolute path to current directory
APP_DIR=$(pwd)

echo "🚀 Starting Nginx + PHP-FPM on port $PORT..."

# Disable default Nginx site to avoid port 80 conflict
if [ -L "/etc/nginx/sites-enabled/default" ]; then
    echo "Disabling default Nginx site..."
    sudo rm /etc/nginx/sites-enabled/default
fi

# Create Nginx configuration
NGINX_CONF="/etc/nginx/sites-available/php-api"
sudo tee "$NGINX_CONF" > /dev/null <<EOF
server {
    listen $PORT;
    server_name _;
    
    root $APP_DIR;
    index index.php;
    
    # Increase worker connections and optimize
    client_max_body_size 10M;
    
    location / {
        try_files \$uri \$uri/ /index.php?\$query_string;
    }
    
    location ~ \.php$ {
        fastcgi_pass unix:/run/php/php8.2-fpm.sock;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME \$document_root\$fastcgi_script_name;
        include fastcgi_params;
        
        # Performance tuning
        fastcgi_buffering on;
        fastcgi_buffer_size 16k;
        fastcgi_buffers 16 16k;
        fastcgi_busy_buffers_size 32k;
    }
    
    # Deny access to hidden files
    location ~ /\. {
        deny all;
    }
}
EOF

# Enable site
sudo ln -sf "$NGINX_CONF" /etc/nginx/sites-enabled/php-api

# Test Nginx configuration
echo "Testing Nginx configuration..."
sudo nginx -t

# Start/restart services
echo "Starting PHP-FPM..."
sudo systemctl restart php8.2-fpm
sudo systemctl enable php8.2-fpm

echo "Starting Nginx..."
sudo systemctl restart nginx
sudo systemctl enable nginx

# Get number of cores and PHP-FPM workers
NUM_CORES=$(nproc)
FPM_WORKERS=$(sudo grep -E "^pm\.(start_servers|max_children)" /etc/php/8.2/fpm/pool.d/www.conf | awk '{print $3}')

echo "✅ Server started successfully!"
echo "📍 URL: http://localhost:$PORT"
echo "⚙️  CPU cores: $NUM_CORES"
echo "� PHP-FPM workers configured:"
sudo grep -E "^pm\." /etc/php/8.2/fpm/pool.d/www.conf | sed 's/^/   /'
echo ""
echo "� Service status:"
sudo systemctl status php8.2-fpm --no-pager -l | head -5
sudo systemctl status nginx --no-pager -l | head -5
