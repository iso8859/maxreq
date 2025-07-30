using System.Diagnostics;
using System.Text;
using System.Text.Json;

namespace ApiLoadTester;

public class LoadTestResult
{
    public TimeSpan TotalDuration { get; set; }
    public int TotalRequests { get; set; }
    public int SuccessfulRequests { get; set; }
    public int FailedRequests { get; set; }
    public double MeanResponseTimeMs { get; set; }
    public double RequestsPerSecond { get; set; }
    public double MinResponseTimeMs { get; set; }
    public double MaxResponseTimeMs { get; set; }
    public List<double> ResponseTimes { get; set; } = new();
}

public class LoginRequest
{
    public string Username { get; set; } = string.Empty;
    public string HashedPassword { get; set; } = string.Empty;
}

public class LoginResponse
{
    public bool Success { get; set; }
    public long? UserId { get; set; }
    public string? ErrorMessage { get; set; }
}

public class ApiLoadTester
{
    private readonly HttpClient _httpClient;
    private readonly string _apiUrl;
    private readonly LoginRequest _testData;

    public ApiLoadTester(string apiUrl, int maxConcurrentConnections = 16)
    {
        // Configure HttpClient with connection pooling
        var handler = new SocketsHttpHandler()
        {
            MaxConnectionsPerServer = maxConcurrentConnections,
            PooledConnectionLifetime = TimeSpan.FromMinutes(2),
            EnableMultipleHttp2Connections = true
        };

        _httpClient = new HttpClient(handler)
        {
            Timeout = TimeSpan.FromSeconds(30)
        };

        _apiUrl = apiUrl;
        _testData = new LoginRequest
        {
            Username = "user1@example.com",
            HashedPassword = "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
        };
    }

    public async Task<LoadTestResult> RunLoadTest(int totalRequests, int maxConcurrency)
    {
        Console.WriteLine($"🚀 Starting load test...");
        Console.WriteLine($"   Target API: {_apiUrl}");
        Console.WriteLine($"   Total Requests: {totalRequests:N0}");
        Console.WriteLine($"   Max Concurrency: {maxConcurrency}");
        Console.WriteLine($"   Connection Pool Size: 16");
        Console.WriteLine();

        var semaphore = new SemaphoreSlim(maxConcurrency, maxConcurrency);
        var tasks = new List<Task<(bool Success, double ResponseTimeMs)>>();
        var stopwatch = Stopwatch.StartNew();

        // Create all tasks
        for (int i = 0; i < totalRequests; i++)
        {
            tasks.Add(ExecuteRequestAsync(semaphore, i + 1));
        }

        // Wait for all tasks to complete
        var results = await Task.WhenAll(tasks);
        stopwatch.Stop();

        // Process results
        var successful = results.Where(r => r.Success).ToList();
        var failed = results.Where(r => !r.Success).ToList();
        var responseTimes = successful.Select(r => r.ResponseTimeMs).ToList();

        var loadTestResult = new LoadTestResult
        {
            TotalDuration = stopwatch.Elapsed,
            TotalRequests = totalRequests,
            SuccessfulRequests = successful.Count,
            FailedRequests = failed.Count,
            ResponseTimes = responseTimes,
            MeanResponseTimeMs = responseTimes.Count > 0 ? responseTimes.Average() : 0,
            MinResponseTimeMs = responseTimes.Count > 0 ? responseTimes.Min() : 0,
            MaxResponseTimeMs = responseTimes.Count > 0 ? responseTimes.Max() : 0,
            RequestsPerSecond = totalRequests / stopwatch.Elapsed.TotalSeconds
        };

        return loadTestResult;
    }

    private async Task<(bool Success, double ResponseTimeMs)> ExecuteRequestAsync(SemaphoreSlim semaphore, int requestId)
    {
        await semaphore.WaitAsync();
        
        try
        {
            var requestStopwatch = Stopwatch.StartNew();
            
            var json = JsonSerializer.Serialize(_testData);
            var content = new StringContent(json, Encoding.UTF8, "application/json");
            
            var response = await _httpClient.PostAsync(_apiUrl, content);
            var responseBody = await response.Content.ReadAsStringAsync();
            
            requestStopwatch.Stop();
            
            // Validate response
            if (response.IsSuccessStatusCode)
            {
                try
                {
                    var loginResponse = JsonSerializer.Deserialize<LoginResponse>(responseBody, new JsonSerializerOptions
                    {
                        PropertyNameCaseInsensitive = true
                    });
                    
                    if (loginResponse?.Success == true && loginResponse.UserId.HasValue)
                    {
                        return (true, requestStopwatch.Elapsed.TotalMilliseconds);
                    }
                }
                catch (JsonException)
                {
                    // JSON parsing failed, but HTTP was successful
                }
            }
            
            return (false, requestStopwatch.Elapsed.TotalMilliseconds);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Request {requestId} failed: {ex.Message}");
            return (false, 0);
        }
        finally
        {
            semaphore.Release();
        }
    }

    public void PrintResults(LoadTestResult result)
    {
        Console.WriteLine("🎯 Load Test Results");
        Console.WriteLine("=" + new string('=', 50));
        Console.WriteLine();
        
        Console.WriteLine($"📊 Overview:");
        Console.WriteLine($"   Total Duration: {result.TotalDuration.TotalSeconds:F2} seconds");
        Console.WriteLine($"   Total Requests: {result.TotalRequests:N0}");
        Console.WriteLine($"   Successful: {result.SuccessfulRequests:N0} ({(double)result.SuccessfulRequests/result.TotalRequests*100:F1}%)");
        Console.WriteLine($"   Failed: {result.FailedRequests:N0} ({(double)result.FailedRequests/result.TotalRequests*100:F1}%)");
        Console.WriteLine();
        
        Console.WriteLine($"⚡ Performance Metrics:");
        Console.WriteLine($"   Requests/Second: {result.RequestsPerSecond:F1}");
        Console.WriteLine($"   Mean Response Time: {result.MeanResponseTimeMs:F1} ms");
        Console.WriteLine($"   Min Response Time: {result.MinResponseTimeMs:F1} ms");
        Console.WriteLine($"   Max Response Time: {result.MaxResponseTimeMs:F1} ms");
        Console.WriteLine();
        
        if (result.ResponseTimes.Count > 0)
        {
            var sortedTimes = result.ResponseTimes.OrderBy(x => x).ToList();
            var p50 = GetPercentile(sortedTimes, 50);
            var p95 = GetPercentile(sortedTimes, 95);
            var p99 = GetPercentile(sortedTimes, 99);
            
            Console.WriteLine($"📈 Response Time Percentiles:");
            Console.WriteLine($"   50th percentile (median): {p50:F1} ms");
            Console.WriteLine($"   95th percentile: {p95:F1} ms");
            Console.WriteLine($"   99th percentile: {p99:F1} ms");
        }
    }

    private double GetPercentile(List<double> sortedValues, int percentile)
    {
        if (sortedValues.Count == 0) return 0;
        
        double index = (percentile / 100.0) * (sortedValues.Count - 1);
        int lowerIndex = (int)Math.Floor(index);
        int upperIndex = (int)Math.Ceiling(index);
        
        if (lowerIndex == upperIndex)
            return sortedValues[lowerIndex];
        
        double weight = index - lowerIndex;
        return sortedValues[lowerIndex] * (1 - weight) + sortedValues[upperIndex] * weight;
    }

    public void Dispose()
    {
        _httpClient?.Dispose();
    }
}

class Program
{
    static async Task Main(string[] args)
    {
        Console.WriteLine("🔥 API Load Tester");
        Console.WriteLine("=" + new string('=', 50));
        Console.WriteLine();

        // Check command line arguments for test size
        int totalRequests = 10000;
        int maxConcurrency = 16;
        string testFilter = "all";
        if (args.Length > 0 && int.TryParse(args[0], out int requestCount))
        {
            totalRequests = requestCount;
        }
        
        if (args.Length > 1 && int.TryParse(args[1], out int concurrency))
        {
            maxConcurrency = concurrency;
        }

        if (args.Length > 2)
        {
            testFilter = args[2].ToLower();
        }

        Console.WriteLine($"Configuration: {totalRequests:N0} requests, {maxConcurrency} concurrent connections");
        Console.WriteLine();

        // Configuration
        const string dotnetApiUrl = "http://localhost:5150/api/auth/get-user-token";
        const string nodeApiUrl = "http://localhost:3000/api/auth/get-user-token";
        const string rustApiUrl = "http://localhost:8080/api/auth/get-user-token";
        const string phpApiUrl = "http://localhost:9000/api/auth/get-user-token";

        if (testFilter == "dotnet" || testFilter == "all")
        {
            await TestApi(".NET API", dotnetApiUrl, totalRequests, maxConcurrency);
        }
        if (testFilter == "node" || testFilter == "all")
        {
            await TestApi("Node.js API", nodeApiUrl, totalRequests, maxConcurrency);
        }
        if (testFilter == "rust" || testFilter == "all")
        {
            await TestApi("Rust API", rustApiUrl, totalRequests, maxConcurrency);
        }
        if (testFilter == "php" || testFilter == "all")
        {
            await TestApi("PHP API", phpApiUrl, totalRequests, maxConcurrency);
        }

        Console.WriteLine();
        Console.WriteLine("✅ Load testing completed!");
    }

    static async Task TestApi(string apiName, string apiUrl, int totalRequests, int maxConcurrency)
    {
        Console.WriteLine($"🎯 Testing {apiName}");
        Console.WriteLine("-" + new string('-', 30));
        
        var tester = new ApiLoadTester(apiUrl, maxConcurrency);
        
        try
        {
            var result = await tester.RunLoadTest(totalRequests, maxConcurrency);
            Console.WriteLine($"✅ {apiName} test completed!");
            Console.WriteLine();
            tester.PrintResults(result);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"❌ {apiName} test failed: {ex.Message}");
            Console.WriteLine($"💡 Make sure the API is running at: {apiUrl}");
        }
        finally
        {
            tester.Dispose();
        }
    }
}
