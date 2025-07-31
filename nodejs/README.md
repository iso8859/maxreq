# UserTokenApi - Node.js

A high-performance Node.js Express API for user authentication using SQLite database with raw SQL queries.

## Features

- **POST /api/auth/get-user-token**: Authenticate user with email and SHA256 hashed password
- **GET /api/auth/create-db**: Generate 10,000 test users in SQLite database
- **GET /health**: Health check endpoint

## 🚀 Deployment Options

### Option 1: Single Node.js Instance (Development)
```bash
# Install dependencies
npm install

# Start the server
npm start
# or
node index.js

# Test the API
curl http://localhost:3000/health
```

### Option 2: Nginx + Multiple Node.js Instances (Production)

For maximum performance, run multiple Node.js processes behind Nginx as a reverse proxy and load balancer.

#### Benefits of Nginx + Node.js Parallelization:
- **Load Distribution**: Spread requests across multiple Node.js processes
- **CPU Utilization**: Leverage all CPU cores effectively
- **High Availability**: If one Node.js process fails, others continue serving
- **Static File Serving**: Nginx efficiently serves static content
- **SSL Termination**: Handle HTTPS at the Nginx level
- **Rate Limiting**: Built-in protection against abuse

## 🔧 Nginx Installation & Configuration

### Windows Installation

**1. Install Nginx**
```bash
# Using Chocolatey
choco install nginx

# Or download from http://nginx.org/en/download.html
# Extract to C:\nginx
```

**2. Install Node.js and Dependencies**
```bash
# Install Node.js
choco install nodejs

# Install dependencies
npm install

# Install PM2 for process management
npm install -g pm2
```

**3. Configure Multiple Node.js Instances with PM2**

Create `ecosystem.config.js`:
```javascript
module.exports = {
  apps: [
    {
      name: 'usertoken-api-1',
      script: 'index.js',
      env: {
        PORT: 3001,
        NODE_ENV: 'production'
      }
    },
    {
      name: 'usertoken-api-2',
      script: 'index.js',
      env: {
        PORT: 3002,
        NODE_ENV: 'production'
      }
    },
    {
      name: 'usertoken-api-3',
      script: 'index.js',
      env: {
        PORT: 3003,
        NODE_ENV: 'production'
      }
    },
    {
      name: 'usertoken-api-4',
      script: 'index.js',
      env: {
        PORT: 3004,
        NODE_ENV: 'production'
      }
    }
  ]
};
```

**4. Configure Nginx Load Balancer**

Edit `C:\nginx\conf\nginx.conf`:
```nginx
events {
    worker_processes auto;
    worker_connections 1024;
}

http {
    include       mime.types;
    default_type  application/octet-stream;
    
    # Upstream configuration for Node.js instances
    upstream nodejs_backend {
        least_conn;  # Load balancing method
        server 127.0.0.1:3001 weight=1 max_fails=3 fail_timeout=30s;
        server 127.0.0.1:3002 weight=1 max_fails=3 fail_timeout=30s;
        server 127.0.0.1:3003 weight=1 max_fails=3 fail_timeout=30s;
        server 127.0.0.1:3004 weight=1 max_fails=3 fail_timeout=30s;
        keepalive 32;
    }
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    
    server {
        listen 80;
        server_name localhost;
        
        # Logging
        access_log logs/nodejs-api.access.log;
        error_log logs/nodejs-api.error.log;
        
        # Security headers
        add_header X-Content-Type-Options nosniff;
        add_header X-Frame-Options DENY;
        add_header X-XSS-Protection "1; mode=block";
        
        # Proxy configuration
        location / {
            limit_req zone=api burst=20 nodelay;
            
            proxy_pass http://nodejs_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection 'upgrade';
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_cache_bypass $http_upgrade;
            
            # Timeouts
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }
        
        # Health check endpoint bypass (for monitoring)
        location /health {
            proxy_pass http://nodejs_backend;
            access_log off;
        }
    }
}
```

### Linux Installation

**Ubuntu/Debian:**
```bash
# Update packages
sudo apt update

# Install Nginx and Node.js
sudo apt install nginx nodejs npm

# Install PM2 globally
sudo npm install -g pm2

# Start and enable Nginx
sudo systemctl start nginx
sudo systemctl enable nginx
```

**CentOS/RHEL:**
```bash
# Install EPEL repository
sudo yum install epel-release

# Install Nginx and Node.js
sudo yum install nginx nodejs npm

# Install PM2
sudo npm install -g pm2

# Start and enable services
sudo systemctl start nginx
sudo systemctl enable nginx
```

**Configure Nginx on Linux (`/etc/nginx/sites-available/nodejs-api`):**
```nginx
upstream nodejs_backend {
    least_conn;
    server 127.0.0.1:3001 weight=1 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3002 weight=1 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3003 weight=1 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:3004 weight=1 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

# Rate limiting
limit_req_zone $binary_remote_addr zone=nodejs_api:10m rate=100r/s;

server {
    listen 80;
    server_name nodejs-api.local localhost;
    
    # Logging
    access_log /var/log/nginx/nodejs-api.access.log;
    error_log /var/log/nginx/nodejs-api.error.log;
    
    # Gzip compression
    gzip on;
    gzip_types text/plain application/json application/javascript text/css;
    
    location / {
        limit_req zone=nodejs_api burst=50 nodelay;
        
        proxy_pass http://nodejs_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Connection pooling
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
        proxy_busy_buffers_size 256k;
    }
    
    # Health monitoring
    location /nginx-health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
```

**Enable the site:**
```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/nodejs-api /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Reload Nginx
sudo systemctl reload nginx
```

## 🚀 Starting the Parallelized Setup

**1. Start multiple Node.js instances:**
```bash
# Using PM2 ecosystem file
pm2 start ecosystem.config.js

# Or manually start instances
pm2 start index.js --name "api-1" -- --port 3001
pm2 start index.js --name "api-2" -- --port 3002
pm2 start index.js --name "api-3" -- --port 3003
pm2 start index.js --name "api-4" -- --port 3004

# Check status
pm2 status
pm2 logs
```

**2. Start Nginx:**
```bash
# Windows
C:\nginx\nginx.exe

# Linux
sudo systemctl start nginx
```

## 📊 Load Balancing Methods

Configure different load balancing algorithms in the `upstream` block:

```nginx
upstream nodejs_backend {
    # Round Robin (default)
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    
    # Least Connections
    least_conn;
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    
    # IP Hash (session persistence)
    ip_hash;
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    
    # Weighted Round Robin
    server 127.0.0.1:3001 weight=3;
    server 127.0.0.1:3002 weight=1;
}
```

## 🧪 Testing the Setup

**1. Test individual Node.js instances:**
```bash
# Test each instance directly
curl http://localhost:3001/health
curl http://localhost:3002/health
curl http://localhost:3003/health
curl http://localhost:3004/health
```

**2. Test load balancer:**
```bash
# Test through Nginx
curl http://localhost/health

# Load test with multiple requests
for i in {1..10}; do curl http://localhost/health; done

# Setup database
curl -X POST http://localhost/api/auth/create-db

# Test authentication
curl -X POST http://localhost/api/auth/get-user-token \
  -H "Content-Type: application/json" \
  -d '{"Username":"user1@example.com","HashedPassword":"ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f"}'
```

## 🔧 Performance Tuning

**PM2 Configuration for Production:**
```bash
# Set PM2 to use cluster mode (automatic scaling)
pm2 start index.js -i max --name "usertoken-api-cluster"

# Or specify number of instances (usually = CPU cores)
pm2 start index.js -i 4 --name "usertoken-api"

# Save PM2 process list
pm2 save

# Setup PM2 startup script
pm2 startup
```

**Nginx Performance Optimizations:**
```nginx
# Add to http block
worker_processes auto;
worker_connections 4096;
keepalive_timeout 65;
client_max_body_size 10M;

# Enable caching
proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=api_cache:10m inactive=60m;
proxy_cache api_cache;
proxy_cache_valid 200 5m;
proxy_cache_use_stale error timeout updating http_500 http_502 http_503 http_504;
```

## 🐛 Troubleshooting

**Common Issues:**

1. **"502 Bad Gateway"**
   - Check if Node.js instances are running: `pm2 status`
   - Verify ports match Nginx upstream configuration
   - Check firewall settings

2. **Uneven load distribution**
   - Use `least_conn` or `ip_hash` load balancing methods
   - Monitor with: `pm2 monit`

3. **High memory usage**
   - Limit PM2 instances: `pm2 start -i 2` instead of `max`
   - Enable Node.js garbage collection: `--max-old-space-size=1024`

**Monitoring Commands:**
```bash
# PM2 monitoring
pm2 status
pm2 logs
pm2 monit

# Nginx monitoring
sudo tail -f /var/log/nginx/access.log
sudo nginx -s reload  # Reload configuration
```

## 🔄 Process Management

**PM2 Commands:**
```bash
# Start/Stop/Restart
pm2 start ecosystem.config.js
pm2 stop all
pm2 restart all
pm2 reload all  # Zero-downtime reload

# Scaling
pm2 scale usertoken-api 6  # Scale to 6 instances

# Monitoring
pm2 logs --lines 50
pm2 flush  # Clear logs
```

This setup provides a production-ready, scalable Node.js deployment that can handle significantly more concurrent requests than a single Node.js instance.

