# Node.js User Token API

A high-performance Node.js API using Express and SQLite with WAL mode for concurrent access.

## Architecture

This API uses **Node.js Cluster Mode** for maximum performance:

- **Why Cluster Mode?**
  - Utilizes all CPU cores 
  - OS-level load balancing across workers (no reverse proxy overhead)
  - Better socket handling with isolated event loops per worker
  - Automatic worker restart on crash
  - Shared port - all workers listen on the same port

- **Database**: SQLite with WAL (Write-Ahead Logging) mode for concurrent reads
- **Connection Pool**: Each worker maintains its own database connection

## Quick Start

### Start the API (Cluster Mode)
```bash
./start-cluster.sh          # Default: port 8080, auto-detect CPU cores
./start-cluster.sh 3000     # Custom port
./start-cluster.sh 3000 4   # Custom port and worker count
```

### Stop the API
```bash
./stop-cluster.sh
```

### View Logs
```bash
tail -f server.log
```

## Endpoints

- **POST** `/nodejs/api/auth/get-user-token` - Authenticate user
  ```json
  {
    "UserName": "user1@example.com",
    "HashedPassword": "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
  }
  ```

- **GET** `/nodejs/api/auth/create-db` - Create 10,000 test users

- **GET** `/nodejshealth` - Health check (returns worker PID)

## Performance

Cluster mode provides the best performance by:
- Eliminating reverse proxy overhead
- Using OS kernel for load balancing
- Running workers on separate CPU cores
- Maintaining persistent database connections

## Alternative Setup (PM2 + Nginx)

If you prefer PM2 with Nginx reverse proxy:
```bash
./setup.sh      # Install and configure
./start.sh      # Start 16 PM2 instances + Nginx on port 8080
./stop.sh       # Stop all services
```

**Note**: Cluster mode is recommended for better performance.
