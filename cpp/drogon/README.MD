# C++ Drogon API Implementation

High-performance user authentication API built with [Drogon](https://github.com/drogonframework/drogon) - a modern C++ web framework.

## Features

- **Fast**: Drogon uses asynchronous I/O for high performance
- **Connection Pool**: Prepared statement pooling for SQLite
- **SQLite Optimizations**: WAL mode, memory-mapped I/O, optimized pragmas
- **Standards Compliant**: Implements the UserToken API specification

## Prerequisites

- C++17 compiler (g++ 7.0+, clang++ 5.0+)
- CMake 3.10+
- Drogon framework
- Libraries: jsoncpp, sqlite3, openssl

## Build & Install

Run the build script to install dependencies and compile:

```bash
./build.sh
```

Or build manually:

```bash
# Install dependencies
sudo apt-get install -y cmake g++ libjsoncpp-dev libsqlite3-dev libssl-dev

# Install Drogon (if not already installed)
cd /tmp
git clone https://github.com/drogonframework/drogon
cd drogon
git submodule update --init
mkdir build && cd build
cmake ..
make -j$(nproc)
sudo make install
sudo ldconfig

# Build this project
cd /home/debian/maxreq/cpp/drogon
mkdir -p build && cd build
cmake ..
make -j$(nproc)
```

## Run

```bash
./build/drogon-api
```

The server will start on `http://0.0.0.0:8080`

## API Endpoints

### Health Check
```bash
GET /api/auth/health
```

### Create Test Database
```bash
GET /api/auth/create-db
```
Creates 10,000 test users in the SQLite database.

### Get User Token
```bash
POST /api/auth/get-user-token
Content-Type: application/json

{
  "UserName": "user1@example.com",
  "HashedPassword": "sha256_hash_of_password"
}
```

Response:
```json
{
  "Success": true,
  "UserId": 1
}
```

## Performance Optimizations

- Prepared statement pooling with mutex-protected statement cache
- SQLite WAL mode for better concurrency
- Memory-mapped I/O (256MB)
- 64MB cache size
- Index on (mail, hashed_password) for fast lookups
- Asynchronous request handling via Drogon
- Optimized compilation flags (-O3, -march=native)

## Architecture

Based on the uWebSockets implementation but using Drogon's HTTP stack:
- Global SQLite connection with prepared statement pool
- Statement reuse for performance
- Thread-safe statement management
- No database connection per request overhead
