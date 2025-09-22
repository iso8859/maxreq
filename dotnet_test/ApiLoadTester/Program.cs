using System.Diagnostics;
using System.Security.Cryptography;
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
    public string UserName { get; set; } = string.Empty;
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
    private readonly JsonSerializerOptions _jsonOptions = new()
    {
        //PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true
    };

    private readonly HttpClient _httpClient;
    private readonly string _apiUrl;

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
    }

    public async Task<LoadTestResult> RunLoadTest(int totalRequests, int maxConcurrency, bool no_db)
    {
        Console.WriteLine($"🚀 Starting load test...");
        Console.WriteLine($"   Target API: {_apiUrl}");
        Console.WriteLine($"   Total Requests: {totalRequests:N0}");
        Console.WriteLine($"   Max Concurrency: {maxConcurrency}");
        Console.WriteLine($"   Connection Pool Size: 16");
        Console.WriteLine();

        string final = _apiUrl + "api/auth/create-db";
        Console.WriteLine($"   Initializing database with: {final}");
        var response = await _httpClient.GetAsync(final);
        var responseBody = await response.Content.ReadAsStringAsync();
        Console.WriteLine($"   Response: {(int)response.StatusCode} {response.StatusCode} {responseBody}");

        var semaphore = new SemaphoreSlim(maxConcurrency, maxConcurrency);
        var tasks = new List<Task<(bool Success, double ResponseTimeMs)>>();
        var stopwatch = Stopwatch.StartNew();

        // Create all tasks
        for (int i = 0; i < totalRequests; i++)
        {
            tasks.Add(ExecuteRequestAsync(semaphore, no_db ? -1 : ((i % 10000) + 1)));
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

    private static string ComputeSha256Hash(string rawData)
    {
        using SHA256 sha256Hash = SHA256.Create();
        byte[] bytes = sha256Hash.ComputeHash(Encoding.UTF8.GetBytes(rawData));

        return Convert.ToHexStringLower(bytes);
    }


    private async Task<(bool Success, double ResponseTimeMs)> ExecuteRequestAsync(SemaphoreSlim semaphore, int requestId)
    {
        await semaphore.WaitAsync();

        try
        {
            var requestStopwatch = Stopwatch.StartNew();

            string final = _apiUrl + "api/auth/get-user-token";
            var testData = new LoginRequest();
            if (requestId == -1)
            {
                testData.UserName = "no_db";
                testData.HashedPassword = "no_db";
            }
            else
            {
                testData.UserName = $"user{requestId}@example.com";
                testData.HashedPassword = ComputeSha256Hash($"password{requestId}");
            }
            var json = JsonSerializer.Serialize(testData, _jsonOptions);
            var content = new StringContent(json, Encoding.UTF8, "application/json");

            var response = await _httpClient.PostAsync(final, content);
            var responseBody = await response.Content.ReadAsStringAsync();
            if (requestId % 1000 == 0)
            {
                Console.WriteLine($"   Request {requestId}: " + responseBody);
            }

            requestStopwatch.Stop();

            // Validate response
            if (!response.IsSuccessStatusCode)
                return (false, requestStopwatch.Elapsed.TotalMilliseconds);

            try
            {
                var loginResponse = JsonSerializer.Deserialize<LoginResponse>(responseBody, _jsonOptions);

                if (loginResponse is { Success: true, UserId: not null })
                    return (true, requestStopwatch.Elapsed.TotalMilliseconds);
                else
                    Console.WriteLine(loginResponse.ErrorMessage);
            }
            catch (JsonException)
            {
                // JSON parsing failed, but HTTP was successful
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
        Console.WriteLine($"   Successful: {result.SuccessfulRequests:N0} ({(double)result.SuccessfulRequests / result.TotalRequests * 100:F1}%)");
        Console.WriteLine($"   Failed: {result.FailedRequests:N0} ({(double)result.FailedRequests / result.TotalRequests * 100:F1}%)");
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

        bool no_db = false;
        if (args.Length > 3)
            no_db = args[3].ToLower() == "no_db";

        Console.WriteLine($"Configuration: {totalRequests:N0} requests, {maxConcurrency} concurrent connections");
        Console.WriteLine();

        // Configuration
        const string dotnetApiUrl = "http://localhost:5000/";
        const string nodeApiUrl = "http://localhost:80/nodejs/";
        const string rustApiUrl = "http://localhost:8080/";
        const string phpApiUrl = "http://localhost:80/php/";
        const string pythonApiUrl = "http://localhost:80/nodejs/";
        const string javaApiUrl = "http://localhost:6000/";
        const string cppApiUrl = "http://localhost:8081/";
        const string goApiUrl = "http://localhost:8082/";
        const string bunApiUrl = "http://localhost:8084/";

        if (testFilter == "dotnet" || testFilter == "all")
        {
            await TestApi(".NET API", dotnetApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "node" || testFilter == "all")
        {
            await TestApi("Node.js API", nodeApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "rust" || testFilter == "all")
        {
            await TestApi("Rust API", rustApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "php" || testFilter == "all")
        {
            await TestApi("PHP API", phpApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "python" || testFilter == "all")
        {
            await TestApi("Python API", pythonApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "java" || testFilter == "all")
        {
            await TestApi("Java API", javaApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "cpp" || testFilter == "all")
        {
            await TestApi("C++ API", cppApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "go" || testFilter == "all")
        {
            await TestApi("Go API", goApiUrl, totalRequests, maxConcurrency, no_db);
        }
        if (testFilter == "bun" || testFilter == "all")
        {
            await TestApi("Bun API", bunApiUrl, totalRequests, maxConcurrency, no_db);
        }

        Console.WriteLine();
        Console.WriteLine("✅ Load testing completed!");
    }

    static async Task TestApi(string apiName, string apiUrl, int totalRequests, int maxConcurrency, bool no_db)
    {
        Console.WriteLine($"🎯 Testing {apiName}");
        Console.WriteLine("-" + new string('-', 30));

        var tester = new ApiLoadTester(apiUrl, maxConcurrency);

        try
        {
            var result = await tester.RunLoadTest(totalRequests, maxConcurrency, no_db);
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
