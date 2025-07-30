# Multi-Language User Token API Comparison

This repository contains seven identical user authentication APIs implemented in different programming languages, along with comprehensive performance testing tools. The project demonstrates performance characteristics across .NET Core, Node.js, Rust, PHP, Python, Java, and C++ implementations.

## 📊 Performance Benchmarks

Based on testing with 100,000 requests and 16 concurrent connections
On an AMD Ryzen 7 2700X, 8 core, 16 logical threads - Windows 10 machine.

```
Rust 17887 req/s
.NET Core 7417 req/s
C++ 5652 req/s (see remarks below)
Java 4526 req/s
Node.js 2076 req/s
Python 1935 req/s
PHP 1227 req/s
```

=> Conclusion : Today if you need performance don't use interpreted code.

Rust is compiled to native code, so it is the fastest.
C++ is also compiled to native code with minimal runtime overhead, expected to perform very well.
Today we would not use C++ for web developement. I already spend a lot of time on some old project to get good HTTP performance, it's difficult and very verbose. I choose to use C# as a wrapper and call C++ from this wrapper, it's simpler and faster to write and to execute. Today Rust is the best choice for C++ wrapper.
.NET Core is also compiled to native code, but it has more overhead than Rust.
Java runs on the JVM with JIT compilation, providing good performance with runtime optimizations.
Node.js, Python, and PHP are interpreted, they are slower but Python with FastAPI/Uvicorn shows good async performance.

Have a look at file result.txt for complete results and cpu.png for CPU load. First yellow part = .NET, middle par = node.js and last yellow part = Rust.

## 📡 API Endpoints

All seven implementations expose identical REST endpoints:

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

#### PHP API
```bash
cd php
php -S localhost:9000 index.php
# Runs on http://localhost:9000
```

#### Python API
```bash
cd python
pip install -r requirements.txt
uvicorn main:app --host 0.0.0.0 --port 7000 --workers 16 
# Runs on http://localhost:7000
```

#### Java API
```bash
cd java
mvn spring-boot:run
# Runs on http://localhost:6000
```

#### C++ API
```bash
cd cpp
vcpkg install
# Build with your preferred method (Visual Studio, CMake, etc.)
./cpp  # or cpp.exe on Windows
# Runs on http://localhost:8081
```

### VS Code Tasks
Use `Ctrl+Shift+P` → "Tasks: Run Task" and select:
- **Run UserTokenApi** - Start .NET API
- **Run Node.js UserTokenApi** - Start Node.js API  
- **Run Rust UserTokenApi** - Start Rust API
- **Run PHP UserTokenApi** - Start PHP API
- **Run Python UserTokenApi** - Start Python API
- **Run Java UserTokenApi** - Start Java API
- **Run C++ UserTokenApi** - Start C++ API
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
- Tests all seven APIs automatically
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
   
   # Terminal 4 - PHP
   cd php && php -S localhost:9000 index.php
   
   # Terminal 5 - Python
   cd python && pip install -r requirements.txt && uvicorn main:app --port 7000 --workers 16
   
   # Terminal 6 - Java
   cd java && mvn spring-boot:run
   
   # Terminal 7 - C++
   cd cpp && vcpkg install && ./cpp
   ```

## 📝 License

This project is for educational and benchmarking purposes.
