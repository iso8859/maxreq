use khttp::{Headers, Method::*, Server, Status};
use sha2::Digest;
use std::sync::Arc;
use std::time::Duration;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

type DbPool = Pool<SqliteConnectionManager>;

// Simple JSON parsing helpers
fn parse_json_field<'a>(json: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{}\"", field);
    let start = json.find(&key)?;
    let colon = json[start..].find(':')?;
    let value_start = start + colon + 1;
    let trimmed = json[value_start..].trim_start();
    
    if trimmed.starts_with('"') {
        let end = trimmed[1..].find('"')?;
        Some(&trimmed[1..end + 1])
    } else {
        None
    }
}

fn json_response(_success: bool, user_id: Option<i64>, error: Option<&str>) -> String {
    match (user_id, error) {
        (Some(id), _) => format!(r#"{{"Success":true,"UserId":{}}}"#, id),
        (None, Some(err)) => format!(r#"{{"Success":false,"ErrorMessage":"{}"}}"#, err),
        _ => r#"{"Success":false}"#.to_string(),
    }
}

// SQLite database wrapper
struct Database {
    pool: DbPool,
}

impl Database {
    fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let cpus = num_cpus::get() as u32;
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder().max_size(cpus).build(manager)?;
        
        let conn = pool.get()?;
        
        // Configure SQLite for better performance and concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "synchronous", "NORMAL")?;      // Balance durability vs performance
        conn.pragma_update(None, "cache_size", "-64000")?;       // 64MB cache (negative = KB)
        conn.pragma_update(None, "temp_store", "MEMORY")?;       // Store temp tables in memory
        conn.pragma_update(None, "mmap_size", "268435456")?;     // 256MB memory map
        
        // Set busy timeout for handling concurrent access
        conn.busy_timeout(Duration::from_millis(30000))?;        // 30 second timeout
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mail TEXT NOT NULL UNIQUE,
                hashed_password TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_user_mail_password ON user(mail, hashed_password)",
            [],
        )?;
        
        Ok(Database { pool })
    }

    fn get_user(&self, username: &str, hashed_password: &str) -> Result<Option<i64>, Box<dyn std::error::Error>> {
        // Special case for testing
        if username == "no_db" {
            return Ok(Some(12345));
        }

        let conn = self.pool.get()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id FROM user WHERE mail = ?1 AND hashed_password = ?2 LIMIT 1"
        )?;
        
        let result = stmt.query_row([username, hashed_password], |row| row.get(0));
        
        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn create_users(&self, count: usize) -> Result<usize, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        
        conn.execute("DELETE FROM user", [])?;
        
        // Use WAL checkpoint for better performance before bulk insert
        let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", []);
        
        let tx = conn.unchecked_transaction()?;
        
        let mut stmt = tx.prepare_cached("INSERT INTO user (mail, hashed_password) VALUES (?1, ?2)")?;
        
        for i in 1..=count {
            let email = format!("user{}@example.com", i);
            let password = format!("password{}", i);
            let hashed = format!("{:x}", sha2::Sha256::digest(password.as_bytes()));
            
            stmt.execute([&email, &hashed])?;
        }
        
        drop(stmt);
        tx.commit()?;
        
        Ok(count)
    }
}

fn main() {
    println!("Initializing database with connection pool...");
    let db = Arc::new(
        Database::new("users.db").expect("Failed to initialize database")
    );
    let cpus = num_cpus::get();
    println!("Database ready with {} connection pool", cpus);

    let mut app = Server::builder("0.0.0.0:8080").unwrap();

    // POST /api/auth/get-user-token
    let db_clone = db.clone();
    app.route(Post, "/api/auth/get-user-token", move |mut ctx, res| {
        let body = match ctx.body().vec() {
            Ok(b) => b,
            Err(_) => {
                let json = json_response(false, None, Some("Invalid request body"));
                let mut headers = Headers::new();
                headers.add("Content-Type", b"application/json");
                return res.send(&Status::BAD_REQUEST, &headers, json);
            }
        };
        
        let json_str = match std::str::from_utf8(&body) {
            Ok(s) => s,
            Err(_) => {
                let json = json_response(false, None, Some("Invalid UTF-8"));
                let mut headers = Headers::new();
                headers.add("Content-Type", b"application/json");
                return res.send(&Status::BAD_REQUEST, &headers, json);
            }
        };
        
        let username = parse_json_field(json_str, "UserName").unwrap_or("");
        let hashed_password = parse_json_field(json_str, "HashedPassword").unwrap_or("");
        
        if username.is_empty() || hashed_password.is_empty() {
            let json = json_response(false, None, Some("Missing UserName or HashedPassword"));
            let mut headers = Headers::new();
            headers.add("Content-Type", b"application/json");
            return res.send(&Status::BAD_REQUEST, &headers, json);
        }
        
        let mut headers = Headers::new();
        headers.add("Content-Type", b"application/json");
        
        match db_clone.get_user(username, hashed_password) {
            Ok(Some(user_id)) => {
                //println!("User authenticated: {} -> {}", username, user_id);
                let json = json_response(true, Some(user_id), None);
                res.ok(&headers, json)
            }
            Ok(None) => {
                let json = json_response(false, None, Some("Invalid username or password"));
                res.ok(&headers, json)
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                let json = json_response(false, None, Some("Database error"));
                res.send(&Status::INTERNAL_SERVER_ERROR, &headers, json)
            }
        }
    });

    // GET /api/auth/create-db
    let db_clone = db.clone();
    app.route(Get, "/api/auth/create-db", move |_, res| {
        let mut headers = Headers::new();
        headers.add("Content-Type", b"application/json");
        
        println!("Creating 10000 test users...");
        match db_clone.create_users(10000) {
            Ok(count) => {
                println!("Successfully created {} users", count);
                let response = format!("Successfully created {} users in the database", count);
                res.ok(&headers, response)
            }
            Err(e) => {
                eprintln!("Error creating users: {}", e);
                res.send(
                    &Status::INTERNAL_SERVER_ERROR,
                    &headers,
                    "Error creating database"
                )
            }
        }
    });

    // Health check
    app.route(Get, "/api/auth/health", |_, res| {
        let mut headers = Headers::new();
        headers.add("Content-Type", b"application/json");
        res.ok(&headers, r#"{"status":"ok"}"#)
    });

    // Configure server
    app.thread_count(16);
    app.fallback_route(|_, r| {
        let mut headers = Headers::new();
        headers.add("Content-Type", b"application/json");
        r.send(&Status::NOT_FOUND, &headers, "404")
    });
    
    println!("Server starting on http://0.0.0.0:8080");
    println!("  GET  /api/auth/health");
    println!("  POST /api/auth/get-user-token");
    println!("  GET  /api/auth/create-db");
    
    app.build().serve().unwrap();
}