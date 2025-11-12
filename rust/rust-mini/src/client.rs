use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    #[serde(rename = "UserName")]
    user_name: String,
    #[serde(rename = "HashedPassword")]
    hashed_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "Success")]
    pub success: bool,
    #[serde(rename = "UserId")]
    pub user_id: Option<i64>,
    #[serde(rename = "ErrorMessage")]
    pub error_message: Option<String>,
}

#[derive(Debug)]
pub struct LoadTestResult {
    pub total_duration: Duration,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub mean_response_time_ms: f64,
    pub requests_per_second: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub response_times: Vec<f64>,
}

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    /// Create a new API client with connection pooling
    pub fn new(base_url: impl Into<String>, max_concurrent_connections: usize) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(max_concurrent_connections)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Hash a password using SHA256
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Health check endpoint
    pub async fn health(&self) -> Result<String> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send health check request")?;

        let text = response
            .text()
            .await
            .context("Failed to read health check response")?;

        Ok(text)
    }

    /// Create database with test users
    pub async fn create_db(&self) -> Result<String> {
        let api_url = self.base_url.replace("%", "");
        let url = format!("{}api/auth/create-db", api_url);
        
        println!("   Initializing database with: {}", url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send create-db request")?;

        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read create-db response")?;

        println!("   Response: {} {} {}", status.as_u16(), status, text);

        if !status.is_success() {
            anyhow::bail!("Failed to create database: HTTP {}", status);
        }

        Ok(text)
    }

    /// Execute a single request
    async fn execute_request(
        &self,
        abs: usize,
        request_id: i32,
    ) -> (bool, f64) {
        let request_stopwatch = Instant::now();

        let mut api_url = self.base_url.clone();
        
        // Handle port rotation logic (matching C# behavior)
        if let Some(start) = self.base_url.find('%') {
            if let Some(end) = self.base_url[start + 1..].find('/') {
                let end = start + 1 + end;
                let port_str = &self.base_url[start + 1..end];
                if let Ok(port) = port_str.parse::<u16>() {
                    let new_port = port + (abs % 16) as u16;
                    api_url = format!(
                        "{}{}{}",
                        &self.base_url[..start],
                        new_port,
                        &self.base_url[end..]
                    );
                }
            }
        }

        let url = format!("{}api/auth/get-user-token", api_url);

        let test_data = if request_id == -1 {
            LoginRequest {
                user_name: "no_db".to_string(),
                hashed_password: "no_db".to_string(),
            }
        } else {
            let username = format!("user{}@example.com", request_id);
            let password = format!("password{}", request_id);
            let hashed_password = Self::hash_password(&password);

            if request_id == 1 {
                let payload = serde_json::to_string(&LoginRequest {
                    user_name: username.clone(),
                    hashed_password: hashed_password.clone(),
                }).unwrap();
                println!("   Sending Request {} to: {}", request_id, url);
                println!("   Payload: {}", payload);
            }

            LoginRequest {
                user_name: username,
                hashed_password,
            }
        };

        match self.client.post(&url).json(&test_data).send().await {
            Ok(response) => {
                let status = response.status();
                match response.text().await {
                    Ok(response_body) => {
                        if request_id as usize % 1000 == 0 {
                            println!("   Request {}: {}", request_id, response_body);
                        }

                        let elapsed = request_stopwatch.elapsed().as_secs_f64() * 1000.0;

                        if !status.is_success() {
                            return (false, elapsed);
                        }

                        match serde_json::from_str::<LoginResponse>(&response_body) {
                            Ok(login_response) => {
                                if login_response.success && login_response.user_id.is_some() {
                                    (true, elapsed)
                                } else {
                                    if let Some(err_msg) = login_response.error_message {
                                        println!("{}", err_msg);
                                    }
                                    (false, elapsed)
                                }
                            }
                            Err(_) => (false, elapsed),
                        }
                    }
                    Err(e) => {
                        println!("Request {} failed to read response: {}", request_id, e);
                        (false, 0.0)
                    }
                }
            }
            Err(e) => {
                println!("Request {} failed: {}", request_id, e);
                (false, 0.0)
            }
        }
    }

    /// Run load test
    pub async fn run_load_test(
        &self,
        total_requests: usize,
        max_concurrency: usize,
        no_db: bool,
    ) -> Result<LoadTestResult> {
        println!("🚀 Starting load test...");
        println!("   Target API: {}", self.base_url);
        println!("   Total Requests: {}", format_number(total_requests));
        println!("   Max Concurrency: {}", max_concurrency);
        println!("   Connection Pool Size: 16");
        println!();

        // Initialize database
        self.create_db().await?;

        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let mut tasks = Vec::new();
        let stopwatch = Instant::now();

        // Create all tasks
        for i in 0..total_requests {
            let sem = semaphore.clone();
            let request_id = if no_db { -1 } else { ((i % 10000) + 1) as i32 };
            let client = ApiClient::new(self.base_url.clone(), 16);

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                client.execute_request(i, request_id).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let results: Vec<(bool, f64)> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        let elapsed = stopwatch.elapsed();

        // Process results
        let successful: Vec<_> = results.iter().filter(|r| r.0).collect();
        let failed: Vec<_> = results.iter().filter(|r| !r.0).collect();
        let response_times: Vec<f64> = successful.iter().map(|r| r.1).collect();

        let load_test_result = LoadTestResult {
            total_duration: elapsed,
            total_requests,
            successful_requests: successful.len(),
            failed_requests: failed.len(),
            response_times: response_times.clone(),
            mean_response_time_ms: if !response_times.is_empty() {
                response_times.iter().sum::<f64>() / response_times.len() as f64
            } else {
                0.0
            },
            min_response_time_ms: response_times.iter().cloned().fold(f64::INFINITY, f64::min),
            max_response_time_ms: response_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            requests_per_second: total_requests as f64 / elapsed.as_secs_f64(),
        };

        Ok(load_test_result)
    }

    /// Print load test results
    pub fn print_results(&self, result: &LoadTestResult) {
        println!("🎯 Load Test Results");
        println!("={}", "=".repeat(50));
        println!();

        println!("📊 Overview:");
        println!("   Total Duration: {:.2} seconds", result.total_duration.as_secs_f64());
        println!("   Total Requests: {}", format_number(result.total_requests));
        println!(
            "   Successful: {} ({:.1}%)",
            format_number(result.successful_requests),
            (result.successful_requests as f64 / result.total_requests as f64) * 100.0
        );
        println!(
            "   Failed: {} ({:.1}%)",
            format_number(result.failed_requests),
            (result.failed_requests as f64 / result.total_requests as f64) * 100.0
        );
        println!();

        println!("⚡ Performance Metrics:");
        println!("   Requests/Second: {:.1}", result.requests_per_second);
        println!("   Mean Response Time: {:.1} ms", result.mean_response_time_ms);
        println!("   Min Response Time: {:.1} ms", result.min_response_time_ms);
        println!("   Max Response Time: {:.1} ms", result.max_response_time_ms);
        println!();

        if !result.response_times.is_empty() {
            let mut sorted_times = result.response_times.clone();
            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let p50 = get_percentile(&sorted_times, 50);
            let p95 = get_percentile(&sorted_times, 95);
            let p99 = get_percentile(&sorted_times, 99);

            println!("📈 Response Time Percentiles:");
            println!("   50th percentile (median): {:.1} ms", p50);
            println!("   95th percentile: {:.1} ms", p95);
            println!("   99th percentile: {:.1} ms", p99);
        }
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    
    result
}

fn get_percentile(sorted_values: &[f64], percentile: i32) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (percentile as f64 / 100.0) * (sorted_values.len() - 1) as f64;
    let lower_index = index.floor() as usize;
    let upper_index = index.ceil() as usize;

    if lower_index == upper_index {
        return sorted_values[lower_index];
    }

    let weight = index - lower_index as f64;
    sorted_values[lower_index] * (1.0 - weight) + sorted_values[upper_index] * weight
}

async fn test_api(
    api_name: &str,
    api_url: &str,
    total_requests: usize,
    max_concurrency: usize,
    no_db: bool,
) {
    println!("🎯 Testing {}", api_name);
    println!("-{}", "-".repeat(30));

    let tester = ApiClient::new(api_url, max_concurrency);

    match tester.run_load_test(total_requests, max_concurrency, no_db).await {
        Ok(result) => {
            println!("✅ {} test completed!", api_name);
            println!();
            tester.print_results(&result);
        }
        Err(e) => {
            println!("❌ {} test failed: {}", api_name, e);
            println!("💡 Make sure the API is running at: {}", api_url);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔥 API Load Tester");
    println!("={}", "=".repeat(50));
    println!();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let total_requests = if args.len() > 1 {
        args[1].parse().unwrap_or(10000)
    } else {
        10000
    };

    let max_concurrency = if args.len() > 2 {
        args[2].parse().unwrap_or(16)
    } else {
        16
    };

    let test_filter = if args.len() > 3 {
        args[3].to_lowercase()
    } else {
        "all".to_string()
    };

    let no_db = if args.len() > 4 {
        args[4].to_lowercase() == "no_db"
    } else {
        false
    };

    println!(
        "Configuration: {} requests, {} concurrent connections for {} tests",
        format_number(total_requests),
        max_concurrency,
        test_filter
    );
    println!();

    // Configuration
    const DOTNET_API_URL: &str = "http://localhost:5000/";
    const NODE_API_URL: &str = "http://localhost:8080/nodejs/";
    const NODE_API_URL2: &str = "http://localhost:8081/nodejs/";
    const NODE_API_URL3: &str = "http://localhost:%3001/nodejs/";
    const RUST_API_URL: &str = "http://localhost:8080/";
    const PHP_API_URL: &str = "http://localhost:8080/php/";
    const PYTHON_API_URL: &str = "http://localhost:8000/nodejs/";
    const JAVA_API_URL: &str = "http://localhost:6000/";
    const CPP_API_URL: &str = "http://localhost:8081/";
    const GO_API_URL: &str = "http://localhost:8082/";
    const BUN_API_URL: &str = "http://localhost:8084/";

    if test_filter == "dotnet" || test_filter == "all" {
        test_api(".NET API", DOTNET_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "node" || test_filter == "all" {
        test_api("Node.js API", NODE_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "node2" || test_filter == "all" {
        test_api("Node.js API", NODE_API_URL2, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "node3" || test_filter == "all" {
        test_api("Node.js API", NODE_API_URL3, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "rust" || test_filter == "all" {
        test_api("Rust API", RUST_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "php" || test_filter == "all" {
        test_api("PHP API", PHP_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "python" || test_filter == "all" {
        test_api("Python API", PYTHON_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "java" || test_filter == "all" {
        test_api("Java API", JAVA_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "cpp" || test_filter == "all" {
        test_api("C++ API", CPP_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "go" || test_filter == "all" {
        test_api("Go API", GO_API_URL, total_requests, max_concurrency, no_db).await;
    }
    if test_filter == "bun" || test_filter == "all" {
        test_api("Bun API", BUN_API_URL, total_requests, max_concurrency, no_db).await;
    }

    println!();
    println!("✅ Load testing completed!");

    Ok(())
}
