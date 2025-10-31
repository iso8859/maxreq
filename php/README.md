For local testing with PHP, you can use the built-in PHP server or set up a more robust environment with PHP-FPM and Nginx.

php -S localhost:80 index.php

**Start Services**
```bash
# Start PHP-FPM
C:\php\php-cgi.exe -b 127.0.0.1:9000

# Start Nginx
C:\nginx\nginx.exe

# Stop Nginx
C:\nginx\nginx.exe -s stop
```

On Linux
```bash
# 1. Setup (one time)
sudo ./setup.sh

# 2. Start server
sudo ./start.sh        # Port 8080
# or
sudo ./start.sh 9000   # Custom port

# 3. Stop server
sudo ./stop.sh
```
