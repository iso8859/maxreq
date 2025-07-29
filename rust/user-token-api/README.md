# UserTokenApi - Rust

A high-performance Rust API for user authentication using SQLite database with raw SQL queries.

## Features

- **POST /api/auth/get-user-token**: Authenticate user with email and SHA256 hashed password
- **GET /api/auth/create-db**: Generate 10,000 test users in SQLite database
- **GET /health**: Health check endpoint

## Technology Stack

- **Axum**: Modern, ergonomic web framework
- **Tokio**: Async runtime for high performance
- **rusqlite**: SQLite database driver with raw SQL
- **serde**: JSON serialization/deserialization
- **SHA2**: Cryptographic hashing
- **Tower**: Middleware and service abstractions

## Database Schema

The application uses a SQLite database (`users.db`) with the following table structure:

```sql
CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mail TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL
)
```

## Installation and Usage

### 1. Build and Run
```bash
cd e:\temp\maxreq\rust\user-token-api
cargo build --release
cargo run
```

### 2. Development Mode
```bash
cargo run
```

The API will be available at `http://localhost:8080`

## API Endpoints

### 1. Get User Token
**POST** `/api/auth/get-user-token`

Authenticates a user by email and SHA256 hashed password.

**Request Body:**
```json
{
  "username": "user1@example.com",
  "hashedPassword": "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
}
```

**Response:**
```json
{
  "success": true,
  "userId": 1,
  "errorMessage": null
}
```

### 2. Create Database
**GET** `/api/auth/create-db`

Creates 10,000 test users in the database. Users are created with emails like `user1@example.com` to `user10000@example.com` and passwords like `password1` to `password10000` (SHA256 hashed).

**Response:**
```
Successfully created 10000 users in the database
```

### 3. Health Check
**GET** `/health`

Simple health check endpoint.

**Response:**
```
UserTokenApi Rust server is running
```

## Test Users

After running the create-db endpoint, you can test with these sample credentials:

- **Email**: `user1@example.com`
- **Password**: `password1`
- **SHA256 Hash**: `0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e`

## Performance Features

### High-Performance Design
- **Async/Await**: Full async support with Tokio runtime
- **Zero-Copy**: Minimal memory allocations
- **Connection Pooling**: SQLite connection management
- **CORS Support**: Cross-origin request handling

### Security Features
- **SHA256 Hashing**: Secure password hashing
- **Input Validation**: Request data validation
- **Error Handling**: Comprehensive error management
- **SQL Injection Protection**: Parameterized queries

## Testing

You can test the API using curl, PowerShell, or any HTTP client:

**Create Database:**
```bash
curl -X GET "http://localhost:8080/api/auth/create-db"
```

**Authenticate User:**
```bash
curl -X POST "http://localhost:8080/api/auth/get-user-token" \
  -H "Content-Type: application/json" \
  -d '{"username":"user1@example.com","hashedPassword":"0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"}'
```

**PowerShell:**
```powershell
# Create database
Invoke-RestMethod -Uri "http://localhost:8080/api/auth/create-db" -Method Get

# Test authentication
$body = @{
    username = "user1@example.com"
    hashedPassword = "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:8080/api/auth/get-user-token" -Method Post -Body $body -ContentType "application/json"
```

## Dependencies

- **tokio**: Async runtime
- **axum**: Web framework
- **serde**: Serialization
- **rusqlite**: SQLite driver
- **sha2**: SHA256 hashing
- **hex**: Hex encoding
- **tower-http**: HTTP middleware
- **tracing**: Logging

## Build Configuration

The project uses Rust 2021 edition with the following key features:
- Async/await support
- Error handling with `Result` types
- Memory safety without garbage collection
- Zero-cost abstractions

This Rust implementation provides the same API contract as the .NET and Node.js versions while leveraging Rust's performance and safety guarantees! 🦀
