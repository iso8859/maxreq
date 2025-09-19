use actix_web::{
    web, App, HttpServer, HttpResponse, Result as ActixResult,
    middleware::Logger,
};
use actix_cors::Cors;
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    #[serde(rename = "hashedPassword")]
    hashed_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    success: bool,
    #[serde(rename = "userId")]
    user_id: Option<i64>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

#[derive(Debug)]
struct User {
    id: i64,
    mail: String,
    hashed_password: String,
}

#[derive(Clone)]
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

    fn get_user_by_credentials(&self, email: &str, hashed_password: &str) -> SqliteResult<Option<User>> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, mail, hashed_password FROM user WHERE mail = ?1 AND hashed_password = ?2"
        )?;
        
        let user_iter = stmt.query_map([email, hashed_password], |row| {
            Ok(User {
                id: row.get(0)?,
                mail: row.get(1)?,
                hashed_password: row.get(2)?,
            })
        })?;

        for user in user_iter {
            return Ok(Some(user?));
        }
        
        Ok(None)
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
    data: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> ActixResult<HttpResponse> {
    match data.get_user_by_credentials(&request.username, &request.hashed_password) {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(LoginResponse {
            success: true,
            user_id: Some(user.id),
            error_message: None,
        })),
        Ok(None) => Ok(HttpResponse::Ok().json(LoginResponse {
            success: false,
            user_id: None,
            error_message: Some("Invalid username or password".to_string()),
        })),
        Err(e) => {
            error!("Database error: {}", e);
            Ok(HttpResponse::Ok().json(LoginResponse {
                success: false,
                user_id: None,
                error_message: Some("An error occurred during authentication".to_string()),
            }))
        }
    }
}

async fn create_db(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    match data.create_test_users(10000) {
        Ok(count) => {
            info!("Created {} test users", count);
            Ok(HttpResponse::Ok().body(format!("Successfully created {} users in the database", count)))
        }
        Err(e) => {
            error!("Failed to create test users: {}", e);
            Ok(HttpResponse::InternalServerError().body("Failed to create test users"))
        }
    }
}

async fn health() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("UserTokenApi Rust server is running"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize application state
    let app_state = AppState::new().map_err(|e| {
        error!("Failed to initialize app state: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    info!("🦀 Rust UserTokenApiActix server running on http://localhost:8080");
    info!("Available endpoints:");
    info!("  POST /api/auth/get-user-token - Authenticate user");
    info!("  GET /api/auth/create-db - Create test database");
    info!("  GET /health - Health check");

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/health", web::get().to(health))
            .route("/api/auth/get-user-token", web::post().to(get_user_token))
            .route("/api/auth/create-db", web::get().to(create_db))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}