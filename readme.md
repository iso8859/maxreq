# Multi-Language User Token API Comparison

This repository contains three identical user authentication APIs implemented in different programming languages, along with comprehensive performance testing tools. The project demonstrates performance characteristics across .NET Core, Node.js, and Rust implementations.

## 📊 Performance Benchmarks

Based on testing with 100,000 requests and 16 concurrent connections
On an AMD Ryzen 7 2700X, 8 core, 16 logical threads - Windows 10 machine.

```
Rust 17887 req/s
.NET Core 7417 req/s
Node.js 2076 req/s
```

=> Conclusion : Today if you need performance don't use interpreted code.

Rust is compiled to native code, so it is the fastest.
.NET Core is also compiled to native code, but it has more overhead than Rust.
Node.js is interpreted, so it is the slowest.

Have a look at file result.txt for complete results and cpu.png for CPU load. First yellow part = .NET, middle par = node.js and last yellow part = Rust.

## 🏗️ Project Structure

```
├── dotnet/UserTokenApi/           # ASP.NET Core Web API (.NET 9.0)
├── nodejs/                       # Express.js API (Node.js)
├── rust/user-token-api/          # Axum-based API (Rust)
├── dotnet_test/ApiLoadTester/    # C# Performance Testing Tool
├── performance-test.ps1          # PowerShell Load Testing Script
└── .vscode/tasks.json            # VS Code Task Definitions
```

## 🚀 API Implementations

### .NET Core API (Port 5150)
- **Framework**: ASP.NET Core 9.0
- **Database**: Raw SQLite with Microsoft.Data.Sqlite
- **Features**: Entity Framework deliberately avoided for performance
- **Authentication**: SHA256 password hashing
- **Location**: `dotnet/UserTokenApi/`

### Node.js API (Port 3000)
- **Framework**: Express.js
- **Database**: SQLite3 with connection pooling
- **Features**: Async/await patterns, built-in crypto module
- **Authentication**: SHA256 password hashing
- **Location**: `nodejs/`

### Rust API (Port 8080)
- **Framework**: Axum with Tokio runtime
- **Database**: rusqlite with Arc<Mutex<Connection>>
- **Features**: Zero-cost abstractions, memory safety
- **Authentication**: SHA256 password hashing (sha2 crate)
- **Location**: `rust/user-token-api/`

## 📡 API Endpoints

All three implementations expose identical REST endpoints:

### Health Check
```http
GET /health
```
Returns: `"UserTokenApi [Language] server is running"`

### User Authentication
```http
POST /api/auth/get-user-token
Content-Type: application/json

{
  "username": "user@example.com",
  "hashedPassword": "sha256_hash_here"
}
```

### Response Format
```json
{
  "success": true,
  "userId": 1,
  "errorMessage": null
}
```

### Database Setup
```http
POST /setup-database/{count}
```
Creates test users in the database (e.g., `/setup-database/10000`)

## 🛠️ Quick Start Commands

### Running the APIs

#### .NET API
```bash
cd dotnet/UserTokenApi
dotnet run
# Runs on http://localhost:5150
```

#### Node.js API
```bash
cd nodejs
npm install
node index.js
# Runs on http://localhost:3000
```

#### Rust API
```bash
cd rust/user-token-api
cargo run
# Runs on http://localhost:8080
```

### VS Code Tasks
Use `Ctrl+Shift+P` → "Tasks: Run Task" and select:
- **Run UserTokenApi** - Start .NET API
- **Run Node.js UserTokenApi** - Start Node.js API  
- **Run Rust UserTokenApi** - Start Rust API
- **Run C# Load Tester** - Execute performance tests
- **Build Rust UserTokenApi** - Compile Rust project

## 🔥 Performance Testing

### C# Load Tester (Recommended)
Enterprise-grade load testing with detailed metrics:

```bash
cd dotnet_test/ApiLoadTester
dotnet run [requests] [concurrency]

# Examples:
dotnet run                    # Default: 10,000 requests, 16 concurrent
dotnet run 1000 8            # 1,000 requests, 8 concurrent
dotnet run 50000 32          # 50,000 requests, 32 concurrent
```

**Features:**
- Tests all three APIs automatically
- Connection pooling optimization
- Detailed percentile analysis (50th, 95th, 99th)
- Request/response validation
- Concurrent request management with SemaphoreSlim

## 🚦 Getting Started

**Start all APIs:**
   ```bash
   # Terminal 1 - .NET
   cd dotnet/UserTokenApi && dotnet run
   
   # Terminal 2 - Node.js  
   cd nodejs && npm install && node index.js
   
   # Terminal 3 - Rust
   cd rust/user-token-api && cargo run
   ```

## 📝 License

This project is for educational and benchmarking purposes.
