#!/bin/bash
# Node.js server setup script for Debian with Nginx load balancing
set -e

cd "$(dirname "$0")"

echo "📦 Installing Node.js, Nginx, and dependencies..."

# Install Node.js (LTS version) using NodeSource repository
if ! command -v node &> /dev/null; then
    echo "Installing Node.js 20 LTS..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi

# Install Nginx if not already installed
if ! command -v nginx &> /dev/null; then
    echo "Installing Nginx..."
    sudo apt-get update
    sudo apt-get install -y nginx
fi

# Verify installations
echo "Node version: $(node --version)"
echo "NPM version: $(npm --version)"
echo "Nginx version: $(nginx -v 2>&1 | grep -o 'nginx/[0-9.]*')"

# Install PM2 globally if not already installed
if ! command -v pm2 &> /dev/null; then
    echo "Installing PM2 globally..."
    sudo npm install -g pm2
fi

# Install project dependencies
echo "Installing project dependencies..."
npm install

# Setup PM2 to start on boot
echo "Configuring PM2 startup..."
sudo env PATH=$PATH:/usr/bin pm2 startup systemd -u $(whoami) --hp $(eval echo ~$(whoami))

# Configure Nginx load balancing
echo "Configuring Nginx load balancer..."

NGINX_CONF="/etc/nginx/sites-available/nodejs-api"
CURRENT_DIR=$(pwd)

# Create Nginx configuration with load balancing
sudo tee "$NGINX_CONF" > /dev/null <<'EOF'
upstream nodejs_backend {
    least_conn;  # Use least connections load balancing algorithm
    
    # All PM2 instances (ports 3001-3016)
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    server 127.0.0.1:3003;
    server 127.0.0.1:3004;
    server 127.0.0.1:3005;
    server 127.0.0.1:3006;
    server 127.0.0.1:3007;
    server 127.0.0.1:3008;
    server 127.0.0.1:3009;
    server 127.0.0.1:3010;
    server 127.0.0.1:3011;
    server 127.0.0.1:3012;
    server 127.0.0.1:3013;
    server 127.0.0.1:3014;
    server 127.0.0.1:3015;
    server 127.0.0.1:3016;
    
    keepalive 64;
}

server {
    listen 8080;
    listen [::]:8080;
    
    server_name _;
    
    # Increase buffer sizes for better performance
    client_body_buffer_size 128k;
    client_max_body_size 10m;
    
    location / {
        proxy_pass http://nodejs_backend;
        proxy_http_version 1.1;
        
        # Headers
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # Buffering
        proxy_buffering off;
        proxy_cache_bypass $http_upgrade;
    }
    
    # Health check endpoint
    location /nginx-health {
        access_log off;
        return 200 "Nginx load balancer is healthy\n";
        add_header Content-Type text/plain;
    }
}
EOF

# Enable the site
sudo ln -sf "$NGINX_CONF" /etc/nginx/sites-enabled/nodejs-api

# Remove default site if it exists (to avoid port conflicts)
sudo rm -f /etc/nginx/sites-enabled/default

# Test Nginx configuration
echo "Testing Nginx configuration..."
sudo nginx -t

echo ""
echo "✅ Installation completed!"
echo "Node.js version: $(node --version)"
echo "PM2 version: $(pm2 --version)"
echo "Nginx version: $(nginx -v 2>&1 | grep -o 'nginx/[0-9.]*')"
echo ""
echo "📊 Configuration:"
echo "  - Nginx load balancer: Port 8080"
echo "  - PM2 instances: Ports 3001-3016 (16 instances)"
echo "  - Load balancing: least_conn algorithm"
echo ""
echo "To start the server: ./start.sh"
