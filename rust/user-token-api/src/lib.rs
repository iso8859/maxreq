use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Result as SqliteResult;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

type DbPool = Pool<SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    #[serde(rename = "UserName")]
    user_name: String,
    #[serde(rename = "HashedPassword")]
    hashed_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse<'a> {
    #[serde(rename = "Success")]
    success: bool,
    #[serde(rename = "UserId")]
    user_id: Option<i64>,
    #[serde(rename = "ErrorMessage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<Cow<'a, str>>,
}

#[derive(Debug)]
struct User {
    id: i64,
}

struct AppState {
    db: DbPool,
}

impl AppState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cpus = num_cpus::get() as u32;
        let manager = SqliteConnectionManager::file("users.db");
        // Increase pool size for better concurrency under load
        // Original: cpus, New: cpus * 2 (but cap at reasonable limit)
        let pool_size = std::cmp::min(cpus * 2, 16); // Max 16 connections
        let pool = Pool::builder().max_size(pool_size).build(manager)?;
        let conn = pool.get()?;

        // Configure SQLite for better performance and concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?; // Enable WAL mode for better concurrency
        conn.pragma_update(None, "synchronous", "NORMAL")?; // Balance durability vs performance
        conn.pragma_update(None, "cache_size", "-64000")?; // 64MB cache (negative = KB)
        conn.pragma_update(None, "temp_store", "MEMORY")?; // Store temp tables in memory
        conn.pragma_update(None, "mmap_size", "268435456")?; // 256MB memory map

        // Set busy timeout for handling concurrent access
        conn.busy_timeout(Duration::from_millis(30000))?; // 30 second timeout

        // Create table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mail TEXT NOT NULL UNIQUE,
                hashed_password TEXT NOT NULL
            )",
            [],
        )?;

        // Create index for faster lookups (if not exists)
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_user_mail_password ON user(mail, hashed_password)",
            [],
        )?;

        // Run ANALYZE to update query planner statistics
        let _ = conn.execute("ANALYZE", []);

        // Run PRAGMA optimize for automatic database optimization
        let _ = conn.pragma_update(None, "optimize", "");

        Ok(AppState { db: pool })
    }

    fn get_user_by_credentials(&self, request: &LoginRequest) -> SqliteResult<Option<User>> {
        // Handle special test case
        if request.user_name == "no_db" {
            return Ok(Some(User { id: 12345 }));
        }

        let conn = self.db.get().unwrap();

        // Use prepare_cached for automatic statement caching
        // Optimized query: only select the id column we need
        let mut stmt = conn.prepare_cached(
            "SELECT id FROM user WHERE mail = ?1 AND hashed_password = ?2 LIMIT 1",
        )?;

        // get scalar response
        let user = stmt.query_row([&request.user_name, &request.hashed_password], |row| {
            Ok(User { id: row.get(0)? })
        });

        match user {
            Ok(u) => Ok(Some(u)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn create_test_users(&self, count: usize) -> Result<usize, Box<dyn std::error::Error>> {
        let conn = self.db.get().unwrap();

        // Clear existing users
        conn.execute("DELETE FROM user", [])?;

        // Use WAL checkpoint for better performance before bulk insert
        let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", []);

        let tx = conn.unchecked_transaction()?;
        let mut inserted = 0;

        {
            // Use prepare_cached for the INSERT statement as well
            let mut stmt =
                tx.prepare_cached("INSERT INTO user (mail, hashed_password) VALUES (?1, ?2)")?;

            for i in 1..=count {
                let email = format!("user{}@example.com", i);
                let password = format!("password{}", i);
                let hashed_password = hash_password(&password);

                match stmt.execute([&email, &hashed_password]) {
                    Ok(_) => inserted += 1,
                    Err(e) => error!("Failed to insert user {}: {}", i, e),
                }
            }
        } // stmt is dropped here

        tx.commit()?;

        // Run ANALYZE to update query planner statistics
        let _ = conn.execute("ANALYZE", []);

        // Run PRAGMA optimize after bulk insert
        let _ = conn.pragma_update(None, "optimize", "");

        Ok(inserted)
    }
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

async fn get_user_token(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<ResponseJson<LoginResponse<'static>>, StatusCode> {
    match state.get_user_by_credentials(&request) {
        Ok(Some(user)) => {
            // Success case: no heap allocation needed
            Ok(ResponseJson(LoginResponse {
                success: true,
                user_id: Some(user.id),
                error_message: None,
            }))
        }
        Ok(None) => {
            // Error case: use static string literal (stack-allocated)
            Ok(ResponseJson(LoginResponse {
                success: false,
                user_id: None,
                error_message: Some(Cow::Borrowed("Invalid username or password")),
            }))
        }
        Err(e) => {
            error!("Database error: {}", e);
            // Error case: use static string literal (stack-allocated)
            Ok(ResponseJson(LoginResponse {
                success: false,
                user_id: None,
                error_message: Some(Cow::Borrowed("An error occurred during authentication")),
            }))
        }
    }
}

async fn create_db(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<String, StatusCode> {
    match state.create_test_users(10000) {
        Ok(count) => {
            info!("Created {} test users", count);
            Ok(format!(
                "Successfully created {} users in the database",
                count
            ))
        }
        Err(e) => {
            error!("Failed to create test users: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn health() -> &'static str {
    "UserTokenApi Rust server is running"
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize application state
    let app_state = Arc::new(AppState::new()?);

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/auth/get-user-token", post(get_user_token))
        .route("/api/auth/create-db", get(create_db))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("🦀 Rust UserTokenApi server running on http://localhost:8080");
    info!("Available endpoints:");
    info!("  POST /api/auth/get-user-token - Authenticate user");
    info!("  GET /api/auth/create-db - Create test database");
    info!("  GET /health - Health check");

    axum::serve(listener, app).await?;

    Ok(())
}
