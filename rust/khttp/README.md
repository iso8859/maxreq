# Rust khttp API Implementation

A simple, high-performance user authentication API built with [khttp](https://github.com/rkarp/khttp) - a minimal, fast HTTP server library for Rust.

## Features

- **Simple**: Minimal dependencies, straightforward code
- **Fast**: khttp is designed for performance with zero-copy parsing
- **SQLite**: Prepared statements with connection pooling via Mutex
- **Standards compliant**: Implements the UserToken API specification

## Prerequisites

- Rust (latest stable)
- Cargo

## Setup

1. Build the application:
```bash
cargo build --release
```

2. Run the server:
```bash
cargo run --release
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

## Testing

1. First, create the test database:
```bash
curl http://localhost:5000/api/auth/create-db
```

2. Test authentication with a valid user:
```bash
curl -X POST http://localhost:5000/api/auth/get-user-token \
  -H "Content-Type: application/json" \
  -d '{"UserName":"user1@example.com","HashedPassword":"0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"}'
```

3. Test the special "no_db" user (bypasses database):
```bash
curl -X POST http://localhost:5000/api/auth/get-user-token \
  -H "Content-Type: application/json" \
  -d '{"UserName":"no_db","HashedPassword":"any"}'
```

## Performance Optimizations

- Uses khttp's efficient zero-copy HTTP parsing
- SQLite prepared statement caching via `prepare_cached()`
- Shared database connection with Mutex for thread safety
- Release build optimizations: LTO, single codegen unit
- 16 worker threads for concurrent request handling

## Why khttp?

khttp is a minimal HTTP server library that focuses on:
- Zero-copy request parsing
- Low memory overhead
- Simple, straightforward API
- No complex async runtime overhead
- Direct control over threading model

Perfect for high-performance APIs where simplicity and speed matter.
