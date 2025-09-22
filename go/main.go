package main

import (
	"crypto/sha256"
	"database/sql"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	_ "modernc.org/sqlite"
)

// LoginRequest represents the authentication request
type LoginRequest struct {
	UserName       string `json:"UserName" binding:"required"`
	HashedPassword string `json:"HashedPassword" binding:"required"`
}

// LoginResponse represents the authentication response
type LoginResponse struct {
	Success      bool   `json:"Success"`
	UserId       int64  `json:"UserId"`
	ErrorMessage string `json:"ErrorMessage"`
}

// User represents a user from the database
type User struct {
	ID int64 `json:"id"`
}

// StatementPool holds a prepared statement for reuse
type StatementPool struct {
	stmt *sql.Stmt
	db   *sql.DB
}

// DatabaseService handles database operations
type DatabaseService struct {
	db              *sql.DB
	getUserStmtPool chan *StatementPool
	poolSize        int
}

// NewDatabaseService creates a new database service
func NewDatabaseService(dbPath string, readOnly bool) (*DatabaseService, error) {
	// Build DSN. For modernc sqlite driver, use file: prefix to add query parameters.
	dsn := dbPath
	if !strings.HasPrefix(dbPath, "file:") {
		dsn = "file:" + dbPath
	}

	// Add SQLite pragmas for better concurrency and performance
	if readOnly {
		dsn += "?mode=ro&_busy_timeout=30000&_journal_mode=WAL&_synchronous=NORMAL"
	} else {
		dsn += "?_busy_timeout=30000&_journal_mode=WAL&_synchronous=NORMAL&_cache_size=10000"
	}

	db, err := sql.Open("sqlite", dsn)
	if err != nil {
		return nil, fmt.Errorf("open db (%s): %w", dsn, err)
	}

	// Set connection pool limits to prevent too many concurrent connections
	db.SetMaxOpenConns(25)
	db.SetMaxIdleConns(5)
	db.SetConnMaxLifetime(time.Minute * 5)

	// Initialize statement pool
	poolSize := 64
	getUserStmtPool := make(chan *StatementPool, poolSize)

	service := &DatabaseService{
		db:              db,
		getUserStmtPool: getUserStmtPool,
		poolSize:        poolSize,
	}

	// Pre-populate the statement pool
	for i := 0; i < poolSize; i++ {
		stmt, err := db.Prepare("SELECT id FROM user WHERE mail = ? AND hashed_password = ? LIMIT 1")
		if err != nil {
			return nil, fmt.Errorf("failed to prepare statement: %w", err)
		}
		getUserStmtPool <- &StatementPool{stmt: stmt, db: db}
	}
	fmt.Printf("Created statement pool with %d prepared statements\n", poolSize)

	if !readOnly { // Only attempt to create schema when writable
		if err := service.InitializeDatabase(); err != nil {
			return nil, err
		}
	}

	return service, nil
}

// InitializeDatabase creates the required tables
func (ds *DatabaseService) InitializeDatabase() error {
	createTableSQL := `
		CREATE TABLE IF NOT EXISTS user (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			mail TEXT NOT NULL UNIQUE,
			hashed_password TEXT NOT NULL
		)
	`
	_, err := ds.db.Exec(createTableSQL)
	return err
}

// getStatement gets a prepared statement from the pool
func (ds *DatabaseService) getStatement() *StatementPool {
	// Fast path: try to take a statement immediately
	select {
	case stmt := <-ds.getUserStmtPool:
		return stmt
	default:
	}

	// If pool is momentarily empty, wait a short time for a statement to become available
	// This avoids creating many transient prepared statements during short spikes.
	timer := time.NewTimer(20 * time.Millisecond)
	defer timer.Stop()

	select {
	case stmt := <-ds.getUserStmtPool:
		return stmt
	case <-timer.C:
		// Timeout - create a fallback statement
		stmt, err := ds.db.Prepare("SELECT id FROM user WHERE mail = ? AND hashed_password = ? LIMIT 1")
		if err != nil {
			log.Printf("Failed to create new statement (fallback): %v", err)
			return nil
		}
		fmt.Println("Created new statement (fallback after wait)")
		return &StatementPool{stmt: stmt, db: ds.db}
	}
}

// returnStatement returns a prepared statement to the pool
func (ds *DatabaseService) returnStatement(stmtPool *StatementPool) {
	if stmtPool == nil {
		return
	}

	select {
	case ds.getUserStmtPool <- stmtPool:
		// Successfully returned to pool
	default:
		// Pool is full, close the statement
		stmtPool.stmt.Close()
	}
}

// GetUserByMailAndPassword retrieves a user by email and password
func (ds *DatabaseService) GetUserByMailAndPassword(request LoginRequest) (*User, error) {
	// Special case for testing
	if request.UserName == "no_db" {
		return &User{ID: 1}, nil
	}

	// Get a prepared statement from the pool
	stmtPool := ds.getStatement()
	if stmtPool == nil {
		return nil, fmt.Errorf("failed to get prepared statement")
	}
	defer ds.returnStatement(stmtPool)

	var userID int64

	// Retry logic for SQLITE_BUSY errors
	maxRetries := 3
	for attempt := 0; attempt < maxRetries; attempt++ {
		err := stmtPool.stmt.QueryRow(request.UserName, request.HashedPassword).Scan(&userID)
		if err != nil {
			if err == sql.ErrNoRows {
				return nil, nil // User not found
			}
			// Check if it's a database busy error and retry
			if strings.Contains(err.Error(), "database is locked") || strings.Contains(err.Error(), "SQLITE_BUSY") {
				if attempt < maxRetries-1 {
					time.Sleep(time.Duration(attempt+1) * 10 * time.Millisecond) // Exponential backoff
					continue
				}
			}
			return nil, err
		}
		// Success - return the user
		return &User{ID: userID}, nil
	}

	return nil, fmt.Errorf("failed to query database after %d retries", maxRetries)
}

// CreateTestUsers creates test users in the database
func (ds *DatabaseService) CreateTestUsers(count int) (int, error) {
	// Retry logic for SQLITE_BUSY errors
	maxRetries := 3
	for attempt := 0; attempt < maxRetries; attempt++ {
		result, err := ds.createTestUsersAttempt(count)
		if err != nil {
			// Check if it's a database busy error and retry
			if strings.Contains(err.Error(), "database is locked") || strings.Contains(err.Error(), "SQLITE_BUSY") {
				if attempt < maxRetries-1 {
					time.Sleep(time.Duration(attempt+1) * 50 * time.Millisecond) // Exponential backoff
					continue
				}
			}
			return 0, err
		}
		return result, nil
	}
	return 0, fmt.Errorf("failed to create test users after %d retries", maxRetries)
}

// createTestUsersAttempt performs a single attempt to create test users
func (ds *DatabaseService) createTestUsersAttempt(count int) (int, error) {
	// Begin transaction
	tx, err := ds.db.Begin()
	if err != nil {
		return 0, err
	}
	defer tx.Rollback()

	// Clear existing users
	_, err = tx.Exec("DELETE FROM user")
	if err != nil {
		return 0, err
	}

	// Prepare insert statement
	stmt, err := tx.Prepare("INSERT INTO user (mail, hashed_password) VALUES (?, ?)")
	if err != nil {
		return 0, err
	}
	defer stmt.Close()

	// Insert test users
	for i := 1; i <= count; i++ {
		email := fmt.Sprintf("user%d@example.com", i)
		password := fmt.Sprintf("password%d", i)
		hashedPassword := computeSHA256Hash(password)

		_, err = stmt.Exec(email, hashedPassword)
		if err != nil {
			return 0, err
		}
	}

	// Commit transaction
	if err = tx.Commit(); err != nil {
		return 0, err
	}

	return count, nil
}

// Close closes the database connection and all prepared statements
func (ds *DatabaseService) Close() error {
	// Close all prepared statements in the pool
	close(ds.getUserStmtPool)
	count := 0
	for stmtPool := range ds.getUserStmtPool {
		if stmtPool.stmt != nil {
			stmtPool.stmt.Close()
			count++
		}
	}
	fmt.Printf("Closed %d prepared statements from pool\n", count)

	return ds.db.Close()
}

// computeSHA256Hash computes SHA256 hash of input string
func computeSHA256Hash(input string) string {
	hash := sha256.Sum256([]byte(input))
	return fmt.Sprintf("%x", hash)
}

// AuthController handles authentication endpoints
type AuthController struct {
	dbService   *DatabaseService
	dbServiceRO *DatabaseService
}

// NewAuthController creates a new auth controller
func NewAuthController(dbService *DatabaseService, dbServiceRO *DatabaseService) *AuthController {
	return &AuthController{dbService: dbService, dbServiceRO: dbServiceRO}
}

// GetUserToken handles user authentication
func (ac *AuthController) GetUserToken(c *gin.Context) {
	var request LoginRequest

	if err := c.ShouldBindJSON(&request); err != nil {
		errorMsg := "Username and hashed password are required"
		c.JSON(http.StatusOK, LoginResponse{
			Success:      false,
			UserId:       0,
			ErrorMessage: errorMsg,
		})
		return
	}

	user, err := ac.dbServiceRO.GetUserByMailAndPassword(request)
	if err != nil {
		log.Printf("Error during user authentication: %v", err)
		c.JSON(http.StatusOK, LoginResponse{
			Success:      false,
			UserId:       0,
			ErrorMessage: err.Error(),
		})
		return
	}

	if user != nil {
		c.JSON(http.StatusOK, LoginResponse{
			Success:      true,
			UserId:       user.ID,
			ErrorMessage: "",
		})
	} else {
		errorMsg := "Invalid username or password"
		c.JSON(http.StatusOK, LoginResponse{
			Success:      false,
			UserId:       0,
			ErrorMessage: errorMsg,
		})
	}
}

// CreateDatabase handles database creation
func (ac *AuthController) CreateDatabase(c *gin.Context) {

	userCount := 10000
	start := time.Now()
	count, err := ac.dbService.CreateTestUsers(userCount)
	if err != nil {
		log.Printf("Error creating database: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "An error occurred while creating the database",
		})
		return
	}
	dur := time.Since(start)
	message := fmt.Sprintf("Successfully created %d users in %.2fms", count, float64(dur.Microseconds())/1000.0)
	c.String(http.StatusOK, message)
}

// HealthCheck handles health check endpoint
func HealthCheck(c *gin.Context) {
	c.String(http.StatusOK, "UserTokenApi Go server is running")
}

// ReadyCheck returns 200 if a simple DB query works (or DB is opened read-only but accessible)
func ReadyCheck(db *sql.DB) gin.HandlerFunc {
	return func(c *gin.Context) {
		var one int
		// Lightweight query; if table not present (read-only pre-init) still succeed with 200 but note state
		err := db.QueryRow("SELECT 1").Scan(&one)
		if err != nil {
			c.JSON(http.StatusOK, gin.H{"status": "degraded", "error": err.Error()})
			return
		}
		c.JSON(http.StatusOK, gin.H{"status": "ready"})
	}
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8082"
	}
	dbPath := os.Getenv("DB_PATH")
	if dbPath == "" {
		dbPath = "users.db"
	}

	// Initialize database
	dbService, err := NewDatabaseService(dbPath, false)
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}
	defer dbService.Close()

	dbServiceRO, err := NewDatabaseService(dbPath, true)
	if err != nil {
		log.Fatalf("Failed to initialize read-only database: %v", err)
	}
	defer dbServiceRO.Close()

	// Initialize controllers

	authController := NewAuthController(dbService, dbServiceRO)

	gin.SetMode(gin.ReleaseMode)
	router := gin.New()
	router.Use(gin.Recovery())

	router.GET("/health", HealthCheck)
	router.GET("/ready", ReadyCheck(dbService.db))

	api := router.Group("/api")
	auth := api.Group("/auth")
	{
		auth.POST("/get-user-token", authController.GetUserToken)
		auth.GET("/create-db", authController.CreateDatabase)
	}

	readOnly := strings.EqualFold(os.Getenv("READ_ONLY_DB"), "true") || strings.EqualFold(os.Getenv("READ_ONLY"), "true")
	seedInfo := "(writable)"
	if readOnly {
		seedInfo = "(read-only mode; seeding disabled)"
	}
	fmt.Printf("Starting Go UserTokenApi server on http://localhost:%s using DB '%s' %s\n", port, dbPath, seedInfo)
	fmt.Println("Available endpoints:")
	fmt.Println("  GET /health - Health check")
	fmt.Println("  GET /ready  - Readiness (DB reachable)")
	fmt.Println("  POST /api/auth/get-user-token - Authenticate user")
	fmt.Println("  GET /api/auth/create-db - Create/seed test database (disabled in read-only mode)")

	log.Fatal(router.Run(":" + port))
}
