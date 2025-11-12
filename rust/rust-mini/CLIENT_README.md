# Rust API Client

API client for consuming the server-tokio.rs (user-token-api) server.

## Features

- **Async HTTP client** using reqwest with rustls (no OpenSSL dependency)
- **SHA256 password hashing** matching server implementation
- **Health check** endpoint
- **Database creation** endpoint
- **User authentication** endpoint
- **Sequential benchmark** testing
- **Concurrent benchmark** testing with configurable concurrency

## Build

```bash
# Build release version
cargo build --release --bin client

# Binary location
./target/release/client
```

## Usage

### Basic Usage

The client runs a full test suite by default:

```bash
./target/release/client
```

This will:
1. Check server health
2. Create database with 10,000 test users
3. Test valid authentication
4. Test invalid credentials
5. Run sequential benchmark (1,000 requests)
6. Run concurrent benchmark (10,000 requests with 10 concurrent workers)

### Prerequisites

Make sure the server is running on `http://localhost:8080`:

```bash
# In the user-token-api directory
cd ../user-token-api
cargo run --release --bin server-tokio
```

## API Client Methods

```rust
use rust_mini::ApiClient;

let client = ApiClient::new("http://localhost:8080");

// Health check
let health = client.health().await?;

// Create database
let result = client.create_db().await?;

// Authenticate user
let response = client.get_user_token("user1@example.com", "password1").await?;
if response.success {
    println!("User ID: {:?}", response.user_id);
}

// Run benchmarks
client.benchmark(1000).await?;
client.benchmark_concurrent(10000, 10).await?;
```

## Test Data

The server creates 10,000 test users with the following pattern:
- Email: `user1@example.com` to `user10000@example.com`
- Password: `password1` to `password10000`

## Benchmark Results

Example output:
```
📊 Concurrent Benchmark Results:
  Total Requests:  10000
  Concurrency:     10
  Successful:      10000
  Failed:          0
  Duration:        2.45s
  Requests/sec:    4081.63
  Avg latency:     2.45ms
```

## Dependencies

- `reqwest` - HTTP client with rustls-tls (pure Rust, no OpenSSL)
- `tokio` - Async runtime
- `serde` - JSON serialization
- `sha2` - SHA256 password hashing
- `anyhow` - Error handling
