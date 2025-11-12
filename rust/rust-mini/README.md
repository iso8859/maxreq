# Rust Mini API

Minimal Rust API equivalent to dotnet-mini for performance comparison.

## Features

- **Axum** minimal web framework
- **Connection Pooling** using `parking_lot::Mutex<Vec<DbPool>>` (equivalent to C#'s `ConcurrentBag`)
- **SQLite** with WAL mode
- **SHA256** password hashing
- 10,000 test users

## Endpoints

- `POST /api/auth/get-user-token` - Authenticate and get token
  ```json
  {
    "mail": "user0@example.com",
    "password": "password0"
  }
  ```

- `GET /api/auth/create-db` - Create database and populate with 10,000 users

## Build and Run

```bash
# Build release version
cargo build --release

# Run
cargo run --release

# Or run the binary directly
./target/release/rust-mini
```

Server listens on port 5000.

## Test

```bash
# Create database
curl http://localhost:5000/api/auth/create-db

# Test login
curl -X POST http://localhost:5000/api/auth/get-user-token \
  -H "Content-Type: application/json" \
  -d '{"mail":"user0@example.com","password":"password0"}'
```

## Performance Optimizations

- WAL mode for SQLite
- Connection pooling with reusable connections
- Release build with LTO and optimizations
- Parking_lot Mutex for faster locking
