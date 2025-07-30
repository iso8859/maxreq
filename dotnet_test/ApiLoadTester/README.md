# API Load Tester

A high-performance C# console application for load testing APIs with concurrent connections.

## Features

- **10,000 parallel requests** with configurable concurrency
- **16 concurrent connections** (configurable)
- **Detailed performance metrics** including percentiles
- **HTTP connection pooling** for optimal performance
- **Real-time progress monitoring**
- **Comprehensive result analysis**

## Performance Metrics

### Core Metrics
- **Requests per Second (RPS)**: Total throughput
- **Mean Response Time**: Average response time in milliseconds
- **Min/Max Response Time**: Fastest and slowest requests
- **Success Rate**: Percentage of successful requests

### Advanced Analytics
- **50th Percentile (Median)**: Typical response time
- **95th Percentile**: Performance under load
- **99th Percentile**: Worst-case scenario handling

## Configuration

The load tester is configured for optimal performance:

```csharp
const int totalRequests = 10000;     // Total number of requests
const int maxConcurrency = 16;       // Concurrent request limit
const int connectionPoolSize = 16;   // HTTP connection pool size
```

## Test Targets

- **.NET API**: `http://localhost:5150/api/auth/get-user-token`
- **Node.js API**: `http://localhost:3000/api/auth/get-user-token`
- **Rust API**: `http://localhost:8080/api/auth/get-user-token`
- **PHP API**: `http://localhost:9000/api/auth/get-user-token`
- **Python API**: `http://localhost:7000/api/auth/get-user-token`
- **Java API**: `http://localhost:6000/api/auth/get-user-token`
- **C++ API**: `http://localhost:8081/api/auth/get-user-token`
- **Go API**: `http://localhost:8082/api/auth/get-user-token`

## Usage

### 1. Build and Run
```bash
cd e:\temp\maxreq\dotnet_test\ApiLoadTester
dotnet build
dotnet run
```

### 2. VS Code Task
Use the VS Code Command Palette:
- Press `Ctrl+Shift+P`
- Type "Tasks: Run Task"
- Select "Run C# Load Tester"

### 3. Make sure APIs are running
Before running the load test, ensure all APIs are running:
- .NET API: `dotnet run --project ../dotnet/UserTokenApi/UserTokenApi.csproj`
- Node.js API: `node ../nodejs/index.js`
- Rust API: `cargo run --manifest-path ../rust/user-token-api/Cargo.toml`
- PHP API: `php -S localhost:9000 ../php/index.php`
- Python API: `uvicorn main:app --host 0.0.0.0 --port 7000 --workers 16 --app-dir ../python`
- Java API: `mvn spring-boot:run --file ../java/pom.xml`
- C++ API: Build and run the C++ application on port 8081
- Go API: `go run main.go --app-dir ../go`

## Sample Output

```
🔥 API Load Tester
==================================================

🎯 Testing .NET API
------------------------------
🚀 Starting load test...
   Target API: http://localhost:5150/api/auth/get-user-token
   Total Requests: 10,000
   Max Concurrency: 16
   Connection Pool Size: 16

✅ .NET API test completed!

🎯 Load Test Results
==================================================

📊 Overview:
   Total Duration: 12.45 seconds
   Total Requests: 10,000
   Successful: 9,987 (99.9%)
   Failed: 13 (0.1%)

⚡ Performance Metrics:
   Requests/Second: 803.2
   Mean Response Time: 18.4 ms
   Min Response Time: 2.1 ms
   Max Response Time: 156.7 ms

📈 Response Time Percentiles:
   50th percentile (median): 15.2 ms
   95th percentile: 34.8 ms
   99th percentile: 67.3 ms
```

## Technical Implementation

### Connection Management
- **SocketsHttpHandler**: Modern HTTP client with connection pooling
- **MaxConnectionsPerServer**: Limits concurrent connections to 16
- **PooledConnectionLifetime**: Recycles connections every 2 minutes
- **HTTP/2 Support**: Enables multiple HTTP/2 connections when available

### Concurrency Control
- **SemaphoreSlim**: Controls concurrent request execution
- **Task.WhenAll**: Waits for all requests to complete
- **Individual timing**: Each request is timed separately

### Error Handling
- **Timeout protection**: 30-second request timeout
- **JSON validation**: Validates API response format
- **Exception capture**: Logs failed requests with details
- **Graceful degradation**: Continues testing despite individual failures

## Dependencies

- **.NET 9.0**: Modern C# features and performance
- **System.Text.Json**: High-performance JSON serialization
- **SocketsHttpHandler**: Advanced HTTP client capabilities

## Performance Optimization

The load tester is optimized for high performance:

1. **Connection Reuse**: HTTP connections are pooled and reused
2. **Async/Await**: Non-blocking I/O operations
3. **Batched Execution**: All requests are queued and executed concurrently
4. **Memory Efficient**: Minimal object allocation during testing
5. **Real-time Metrics**: Low-overhead performance measurement

This load tester provides enterprise-grade performance testing capabilities for your APIs! 🚀
