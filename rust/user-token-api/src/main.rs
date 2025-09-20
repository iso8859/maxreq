use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    #[serde(rename = "UserName")]
    user_name: String,
    #[serde(rename = "HashedPassword")]
    hashed_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    #[serde(rename = "Success")]
    success: bool,
    #[serde(rename = "UserId")]
    user_id: Option<i64>,
    #[serde(rename = "ErrorMessage")]
    error_message: Option<String>,
}

#[derive(Debug)]
struct User {
    id: i64,
    mail: String,
    hashed_password: String,
}

struct AppState {
    db: Arc<Mutex<Connection>>,
}

impl AppState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open("users.db")?;
        
        // Create table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mail TEXT NOT NULL UNIQUE,
                hashed_password TEXT NOT NULL
            )",
            [],
        )?;

        Ok(AppState {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    fn get_user_by_credentials(&self, request: &LoginRequest) -> SqliteResult<Option<User>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT id, mail, hashed_password FROM user WHERE mail = ?1 AND hashed_password = ?2 limit 1"
        )?;

        // get scalar response
        let user = stmt.query_row([&request.user_name, &request.hashed_password], |row| {
            Ok(User {
                id: row.get(0)?,
                mail: row.get(1)?,
                hashed_password: row.get(2)?,
            })
        });
      
        match user {
            Ok(u) => Ok(Some(u)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e)
        }

    }

    fn create_test_users(&self, count: usize) -> Result<usize, Box<dyn std::error::Error>> {
        let conn = self.db.lock().unwrap();
        
        // Clear existing users
        conn.execute("DELETE FROM user", [])?;
        
        let tx = conn.unchecked_transaction()?;
        let mut inserted = 0;
        
        {
            let mut stmt = tx.prepare("INSERT INTO user (mail, hashed_password) VALUES (?1, ?2)")?;
            
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
) -> Result<ResponseJson<LoginResponse>, StatusCode> {

    match state.get_user_by_credentials(&request) {
        Ok(Some(user)) => Ok(ResponseJson(LoginResponse {
            success: true,
            user_id: Some(user.id),
            error_message: None,
        })),
        Ok(None) => Ok(ResponseJson(LoginResponse {
            success: false,
            user_id: None,
            error_message: Some("Invalid username or password".to_string()),
        })),
        Err(e) => {
            error!("Database error: {}", e);
            Ok(ResponseJson(LoginResponse {
                success: false,
                user_id: None,
                error_message: Some("An error occurred during authentication".to_string()),
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
            Ok(format!("Successfully created {} users in the database", count))
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
