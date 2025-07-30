# Go Installation and Benchmark Instructions

## Install Go

### Windows (Recommended method):
1. **Download Go**: Visit https://golang.org/dl/
2. **Download**: Go 1.21+ for Windows (amd64)
3. **Install**: Run the .msi installer
4. **Verify**: Open new PowerShell and run: `go version`

### Alternative - Using Chocolatey:
```powershell
# Install Chocolatey first if not installed
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install Go
choco install golang
```

### Alternative - Using Winget:
```powershell
winget install GoLang.Go
```

## Run Go Benchmark

Once Go is installed, follow these steps:

### 1. Start Go API
```powershell
cd E:\temp\maxreq\go
go mod tidy
go run main.go
```

The Go API will start on http://localhost:8082

### 2. Create Test Database
In another terminal:
```powershell
curl http://localhost:8082/api/auth/create-db
```

### 3. Run Load Test
```powershell
cd E:\temp\maxreq\dotnet_test\ApiLoadTester
dotnet run go
```

## Expected Results

Based on Go's performance characteristics, you should see:
- **8,000-12,000 requests/second**
- **Low response times** (5-15ms average)
- **High success rate** (99%+)
- **Good percentile performance**

## Full Benchmark (All APIs)

To test all 8 APIs including Go:
```powershell
dotnet run all
```

## Performance Comparison

Go should rank approximately:
1. **Rust**: ~17,887 req/s
2. **Go**: ~10,000+ req/s (NEW!)
3. **.NET Core**: ~7,417 req/s
4. **C++**: ~5,652 req/s
5. **Java**: ~4,526 req/s
6. **Node.js**: ~2,076 req/s
7. **Python**: ~1,935 req/s
8. **PHP**: ~1,227 req/s
