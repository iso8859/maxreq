# Simple API Performance Test
param(
    [int]$RequestCount = 5
)

$testBody = @{
    username = "user1@example.com"
    hashedPassword = "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
} | ConvertTo-Json

Write-Host "🏁 API Performance Test" -ForegroundColor Magenta
Write-Host ("=" * 40) -ForegroundColor Magenta

# Test .NET API
Write-Host "`n🔍 Testing .NET API..." -ForegroundColor Cyan
try {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $dotnetResponse = Invoke-RestMethod -Uri "http://localhost:5150/api/auth/get-user-token" -Method Post -Body $testBody -ContentType "application/json"
    $stopwatch.Stop()
    Write-Host "✅ .NET API: $($stopwatch.ElapsedMilliseconds)ms" -ForegroundColor Green
    $dotnetTime = $stopwatch.ElapsedMilliseconds
}
catch {
    Write-Host "❌ .NET API failed: $($_.Exception.Message)" -ForegroundColor Red
    $dotnetTime = -1
}

# Test Node.js API
Write-Host "`n🔍 Testing Node.js API..." -ForegroundColor Cyan
try {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $nodeResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/auth/get-user-token" -Method Post -Body $testBody -ContentType "application/json"
    $stopwatch.Stop()
    Write-Host "✅ Node.js API: $($stopwatch.ElapsedMilliseconds)ms" -ForegroundColor Green
    $nodeTime = $stopwatch.ElapsedMilliseconds
}
catch {
    Write-Host "❌ Node.js API failed: $($_.Exception.Message)" -ForegroundColor Red
    $nodeTime = -1
}

# Load test
Write-Host "`n⚡ Load Testing ($RequestCount requests each)..." -ForegroundColor Yellow

# .NET Load Test
$dotnetTimes = @()
$dotnetTotal = Measure-Command {
    for ($i = 1; $i -le $RequestCount; $i++) {
        try {
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            $response = Invoke-RestMethod -Uri "http://localhost:5150/api/auth/get-user-token" -Method Post -Body $testBody -ContentType "application/json" -ErrorAction Stop
            $sw.Stop()
            $dotnetTimes += $sw.ElapsedMilliseconds
        }
        catch {
            # Ignore errors for load test
        }
    }
}

# Node.js Load Test
$nodeTimes = @()
$nodeTotal = Measure-Command {
    for ($i = 1; $i -le $RequestCount; $i++) {
        try {
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            $response = Invoke-RestMethod -Uri "http://localhost:3000/api/auth/get-user-token" -Method Post -Body $testBody -ContentType "application/json" -ErrorAction Stop
            $sw.Stop()
            $nodeTimes += $sw.ElapsedMilliseconds
        }
        catch {
            # Ignore errors for load test
        }
    }
}

# Results
Write-Host "`n🎯 Results:" -ForegroundColor Green
Write-Host ("=" * 40) -ForegroundColor Green

if ($dotnetTime -gt 0 -and $nodeTime -gt 0) {
    Write-Host "Single Request:" -ForegroundColor White
    Write-Host "  .NET: ${dotnetTime}ms" -ForegroundColor White
    Write-Host "  Node.js: ${nodeTime}ms" -ForegroundColor White
    $winner = if ($dotnetTime -lt $nodeTime) { ".NET" } else { "Node.js" }
    Write-Host "  Winner: $winner" -ForegroundColor Yellow
}

if ($dotnetTimes.Count -gt 0) {
    $dotnetAvg = [math]::Round(($dotnetTimes | Measure-Object -Average).Average)
    $dotnetRps = [math]::Round($dotnetTimes.Count / $dotnetTotal.TotalSeconds)
    Write-Host "`n.NET Load Test:" -ForegroundColor White
    Write-Host "  Average: ${dotnetAvg}ms" -ForegroundColor White
    Write-Host "  Requests/sec: $dotnetRps" -ForegroundColor White
}

if ($nodeTimes.Count -gt 0) {
    $nodeAvg = [math]::Round(($nodeTimes | Measure-Object -Average).Average)
    $nodeRps = [math]::Round($nodeTimes.Count / $nodeTotal.TotalSeconds)
    Write-Host "`nNode.js Load Test:" -ForegroundColor White
    Write-Host "  Average: ${nodeAvg}ms" -ForegroundColor White
    Write-Host "  Requests/sec: $nodeRps" -ForegroundColor White
}

Write-Host "`n💡 Both APIs should be running for accurate comparison" -ForegroundColor Gray
