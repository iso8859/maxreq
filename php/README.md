# PHP UserToken API

A high-performance PHP implementation of the UserToken API using:
- **PHP 8.0+** with built-in development server or Nginx
- **SQLite with PDO** for database operations
- **JSON REST API** with identical contract to other implementations
- **Optimized batch database operations** for performance
- **Zero external dependencies** - uses only PHP standard library

## 🚀 Quick Start Options

### Option 1: PHP Built-in Development Server (Simplest)
```bash
# Start the built-in PHP server
php -S localhost:8080 index.php

# Test the API
curl http://localhost:8080/health
```

### Option 2: Nginx + PHP-FPM (Production-ready)

#### Windows Installation

**1. Install Nginx**
```bash
# Download from http://nginx.org/en/download.html
# Extract to C:\nginx
# Or use Chocolatey:
choco install nginx

# Or use Scoop:
scoop install nginx
```

**2. Install PHP and PHP-FPM**
```bash
# Download PHP from https://windows.php.net/download/
# Extract to C:\php
# Or use Chocolatey:
choco install php

# Enable PHP-FPM by editing C:\php\php.ini:
# Uncomment: extension=pdo_sqlite
# Uncomment: extension=json
```

**3. Configure PHP-FPM**

Create `C:\php\php-fpm.conf`:
```ini
[global]
pid = C:/php/php-fpm.pid
error_log = C:/php/php-fpm.log

[www]
listen = 127.0.0.1:9000
listen.allowed_clients = 127.0.0.1
pm = dynamic
pm.max_children = 5
pm.start_servers = 2
pm.min_spare_servers = 1
pm.max_spare_servers = 3
```

**4. Configure Nginx**

Edit `C:\nginx\conf\nginx.conf`:
```nginx
events {
    worker_connections 1024;
}

http {
    include       mime.types;
    default_type  application/octet-stream;
    
    server {
        listen 80;
        server_name localhost;
        root E:/temp/maxreq/php;
        index index.php;
        
        # Handle API routes
        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }
        
        # PHP-FPM configuration
        location ~ \.php$ {
            fastcgi_pass   127.0.0.1:9000;
            fastcgi_index  index.php;
            fastcgi_param  SCRIPT_FILENAME $document_root$fastcgi_script_name;
            include        fastcgi_params;
        }
        
        # Security headers
        add_header X-Content-Type-Options nosniff;
        add_header X-Frame-Options DENY;
        add_header X-XSS-Protection "1; mode=block";
    }
}
```

**5. Start Services**
```bash
# Start PHP-FPM
C:\php\php-cgi.exe -b 127.0.0.1:9000

# Start Nginx
C:\nginx\nginx.exe

# Stop Nginx
C:\nginx\nginx.exe -s stop
```

#### Linux Installation

**Ubuntu/Debian:**
```bash
# Update package list
sudo apt update

# Install Nginx and PHP-FPM
sudo apt install nginx php-fpm php-sqlite3 php-json

# Start and enable services
sudo systemctl start nginx
sudo systemctl enable nginx
sudo systemctl start php8.2-fpm
sudo systemctl enable php8.2-fpm
```

**CentOS/RHEL:**
```bash
# Install EPEL repository
sudo yum install epel-release

# Install Nginx and PHP-FPM
sudo yum install nginx php-fpm php-pdo

# Start and enable services
sudo systemctl start nginx
sudo systemctl enable nginx
sudo systemctl start php-fpm
sudo systemctl enable php-fpm
```

**Configure Nginx Virtual Host (`/etc/nginx/sites-available/php-api`):**
```nginx
server {
    listen 80;
    server_name php-api.local localhost;
    root /path/to/maxreq/php;
    index index.php;
    
    # Logging
    access_log /var/log/nginx/php-api.access.log;
    error_log /var/log/nginx/php-api.error.log;
    
    # Handle API routes
    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }
    
    # PHP-FPM configuration
    location ~ \.php$ {
        fastcgi_pass unix:/var/run/php/php8.2-fpm.sock;
        fastcgi_index index.php;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }
    
    # Security
    location ~ /\.ht {
        deny all;
    }
    
    # Performance optimizations
    location ~* \.(jpg|jpeg|png|gif|ico|css|js)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

**Enable the site:**
```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/php-api /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Reload Nginx
sudo systemctl reload nginx
```

#### macOS Installation

**Using Homebrew:**
```bash
# Install Nginx and PHP
brew install nginx php

# Start services
brew services start nginx
brew services start php

# PHP-FPM will be available at /opt/homebrew/var/run/php-fpm.sock
```

## ⚙️ PHP-FPM Configuration

**Optimize PHP-FPM for performance (`/etc/php/8.2/fpm/pool.d/www.conf`):**
```ini
[www]
user = www-data
group = www-data
listen = /var/run/php/php8.2-fpm.sock
listen.owner = www-data
listen.group = www-data

# Process management
pm = dynamic
pm.max_children = 50
pm.start_servers = 5
pm.min_spare_servers = 5
pm.max_spare_servers = 35
pm.max_requests = 500

# Performance tuning
request_slowlog_timeout = 5s
slowlog = /var/log/php8.2-fpm.log.slow
```

## 🧪 Testing the Setup

**1. Test Nginx is running:**
```bash
# Check Nginx status
sudo systemctl status nginx

# Check if port 80 is listening
sudo netstat -tlnp | grep :80
```

**2. Test PHP-FPM:**
```bash
# Check PHP-FPM status
sudo systemctl status php8.2-fpm

# Test PHP-FPM socket
sudo ls -la /var/run/php/
```

**3. Test the API:**
```bash
# Health check
curl http://localhost/health

# Setup database
curl -X POST http://localhost/setup-database/1000

# Test authentication
curl -X POST http://localhost/api/auth/get-user-token \
  -H "Content-Type: application/json" \
  -d '{"Username":"user1@example.com","HashedPassword":"ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f"}'
```

## 🐛 Troubleshooting

**Common Issues:**

1. **"502 Bad Gateway"**
   - Check PHP-FPM is running: `sudo systemctl status php-fpm`
   - Verify socket path in Nginx config matches PHP-FPM config
   - Check PHP-FPM logs: `/var/log/php-fpm/www-error.log`

2. **"File not found" errors**
   - Verify document root path in Nginx config
   - Check file permissions: `sudo chown -R www-data:www-data /path/to/project`
   - Ensure `try_files` directive is correct

3. **"Permission denied"**
   - Check SELinux (CentOS/RHEL): `sudo setsebool -P httpd_can_network_connect 1`
   - Verify Nginx user has access: `sudo usermod -a -G www-data nginx`

4. **"Database not writable"**
   - Ensure web server user can write to directory
   - Set proper permissions: `sudo chmod 755 /path/to/project && sudo chmod 666 users.db`

**Performance Optimization:**
```nginx
# Add to http block in nginx.conf
gzip on;
gzip_types text/plain application/json;
keepalive_timeout 65;
client_max_body_size 10M;

# Enable FastCGI caching
fastcgi_cache_path /var/cache/nginx levels=1:2 keys_zone=php_cache:10m inactive=60m;
fastcgi_cache php_cache;
fastcgi_cache_valid 200 10m;
```

## 🔄 Service Management

**Start/Stop Services:**
```bash
# Nginx
sudo systemctl start nginx
sudo systemctl stop nginx
sudo systemctl restart nginx
sudo systemctl reload nginx

# PHP-FPM
sudo systemctl start php8.2-fpm
sudo systemctl stop php8.2-fpm
sudo systemctl restart php8.2-fpm

# Check logs
sudo tail -f /var/log/nginx/error.log
sudo tail -f /var/log/php8.2-fpm.log
```

