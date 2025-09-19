# Go User Token API

A high-performance Go implementation of the User Token API using Gin framework and SQLite.

## Features

- **High Performance**: Built with Gin framework for optimal HTTP performance
- **SQLite Integration**: Direct SQLite3 database access with prepared statements
- **Concurrent Safe**: Thread-safe database operations with proper connection handling
- **Memory Efficient**: Minimal allocations and efficient JSON serialization
- **SHA-256 Hashing**: Secure password hashing using Go's crypto package

## Performance Characteristics

Go is expected to perform very well in this benchmark due to:
- **Compiled to native code** with Go's efficient runtime
- **Goroutines**: Lightweight concurrency for handling multiple requests
- **Gin framework**: High-performance HTTP router with minimal overhead
- **Direct SQLite access**: No ORM overhead, using prepared statements
- **Efficient memory management**: Go's garbage collector optimized for server workloads

## API Endpoints

### Health Check
**GET** `/health`

Returns: `"UserTokenApi Go server is running"`

### User Authentication
**POST** `/api/auth/get-user-token`

Request:
```json
{
  "username": "user@example.com",
  "hashedPassword": "sha256_hash_here"
}
```

Response:
```json
{
  "success": true,
  "userId": 1,
  "errorMessage": null
}
```

### Database Setup
**GET** `/api/auth/create-db`

Creates 10,000 test users in the SQLite database.

## Running the API

### Prerequisites
- Go 1.21 or later
- CGO enabled (for SQLite support)

### Build and Run
```bash
cd go
go mod tidy
go run main.go
```

The server will start on `http://localhost:8082`

### Build for Production
```bash
go build -o user-token-api main.go
./user-token-api
```

## Testing

### Health Check
```bash
curl http://localhost:8082/health
```

### Create Test Database
```bash
curl http://localhost:8082/api/auth/create-db
```

### Test Authentication
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"username":"user1@example.com","hashedPassword":"ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f"}' \
  http://localhost:8082/api/auth/get-user-token
```

## Architecture

- **main.go**: Complete single-file implementation
- **Gin Router**: HTTP routing and middleware
- **SQLite3 Driver**: Database operations with prepared statements
- **JSON Binding**: Automatic request/response JSON handling
- **Error Handling**: Comprehensive error handling and logging

## Dependencies

- `github.com/gin-gonic/gin`: High-performance HTTP framework
- `github.com/mattn/go-sqlite3`: Pure Go SQLite3 driver

## Performance Optimizations

1. **Gin Release Mode**: Optimized for production performance
2. **Prepared Statements**: Reusable SQL statements for better performance
3. **Transaction Management**: Bulk operations use database transactions
4. **Minimal Middleware**: Only essential middleware for maximum throughput
5. **Direct JSON Binding**: Efficient request/response serialization

Expected performance: **8,000-12,000 requests/second** based on Go's efficient runtime and Gin's performance characteristics.
