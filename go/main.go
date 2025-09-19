package main

import (
	"crypto/sha256"
	"database/sql"
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	_ "modernc.org/sqlite"
)

// LoginRequest represents the authentication request
type LoginRequest struct {
	Username       string `json:"username" binding:"required"`
	HashedPassword string `json:"hashedPassword" binding:"required"`
}

// LoginResponse represents the authentication response
type LoginResponse struct {
	Success      bool    `json:"success"`
	UserId       *int64  `json:"userId"`
	ErrorMessage *string `json:"errorMessage"`
}

// User represents a user from the database
type User struct {
	ID int64 `json:"id"`
}

// DatabaseService handles database operations
type DatabaseService struct {
	db *sql.DB
}

// NewDatabaseService creates a new database service
func NewDatabaseService(dbPath string) (*DatabaseService, error) {
	readOnly := strings.EqualFold(os.Getenv("READ_ONLY_DB"), "true") || strings.EqualFold(os.Getenv("READ_ONLY"), "true")

	// Build DSN. For modernc sqlite driver, use file: prefix to add query parameters.
	dsn := dbPath
	if !strings.HasPrefix(dbPath, "file:") {
		dsn = "file:" + dbPath
	}
	if readOnly {
		dsn += "?mode=ro" // open database in read-only mode
	} else {
		// Busy timeout + shared cache can help under load; pragmas applied after open.
		if !strings.Contains(dsn, "?") {
			dsn += "?"
		} else {
			dsn += "&"
		}
		dsn += "_busy_timeout=5000"
	}

	db, err := sql.Open("sqlite", dsn)
	if err != nil {
		return nil, fmt.Errorf("open db (%s): %w", dsn, err)
	}

	// Apply performance pragmas only if writable
	if !readOnly {
		pragmas := []string{
			"PRAGMA journal_mode=WAL;",
			"PRAGMA synchronous=OFF;",
			"PRAGMA temp_store=MEMORY;",
			"PRAGMA mmap_size=134217728;", // 128MB
		}
		for _, p := range pragmas {
			if _, perr := db.Exec(p); perr != nil {
				log.Printf("warning: failed to apply pragma %s: %v", p, perr)
			}
		}
	}

	service := &DatabaseService{db: db}
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

// GetUserByMailAndPassword retrieves a user by email and password
func (ds *DatabaseService) GetUserByMailAndPassword(mail, hashedPassword string) (*User, error) {
	query := "SELECT id FROM user WHERE mail = ? AND hashed_password = ?"
	var userID int64
	
	err := ds.db.QueryRow(query, mail, hashedPassword).Scan(&userID)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil // User not found
		}
		return nil, err
	}

	return &User{ID: userID}, nil
}

// CreateTestUsers creates test users in the database
func (ds *DatabaseService) CreateTestUsers(count int) (int, error) {
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

// Close closes the database connection
func (ds *DatabaseService) Close() error {
	return ds.db.Close()
}

// computeSHA256Hash computes SHA256 hash of input string
func computeSHA256Hash(input string) string {
	hash := sha256.Sum256([]byte(input))
	return fmt.Sprintf("%x", hash)
}

// AuthController handles authentication endpoints
type AuthController struct {
	dbService *DatabaseService
}

// NewAuthController creates a new auth controller
func NewAuthController(dbService *DatabaseService) *AuthController {
	return &AuthController{dbService: dbService}
}

// GetUserToken handles user authentication
func (ac *AuthController) GetUserToken(c *gin.Context) {
	var request LoginRequest
	
	if err := c.ShouldBindJSON(&request); err != nil {
		errorMsg := "Username and hashed password are required"
		c.JSON(http.StatusOK, LoginResponse{
			Success:      false,
			UserId:       nil,
			ErrorMessage: &errorMsg,
		})
		return
	}

	user, err := ac.dbService.GetUserByMailAndPassword(request.Username, request.HashedPassword)
	if err != nil {
		log.Printf("Error during user authentication: %v", err)
		errorMsg := "An error occurred during authentication"
		c.JSON(http.StatusOK, LoginResponse{
			Success:      false,
			UserId:       nil,
			ErrorMessage: &errorMsg,
		})
		return
	}

	if user != nil {
		c.JSON(http.StatusOK, LoginResponse{
			Success:      true,
			UserId:       &user.ID,
			ErrorMessage: nil,
		})
	} else {
		errorMsg := "Invalid username or password"
		c.JSON(http.StatusOK, LoginResponse{
			Success:      false,
			UserId:       nil,
			ErrorMessage: &errorMsg,
		})
	}
}

// CreateDatabase handles database creation
func (ac *AuthController) CreateDatabase(c *gin.Context) {
	// Prevent seeding if DB opened read-only
	if strings.EqualFold(os.Getenv("READ_ONLY_DB"), "true") || strings.EqualFold(os.Getenv("READ_ONLY"), "true") {
		c.JSON(http.StatusForbidden, gin.H{"error": "Database opened in read-only mode"})
		return
	}

	userCount := 10000
	if v := os.Getenv("SEED_USER_COUNT"); v != "" {
		if parsed, err := strconv.Atoi(v); err == nil && parsed > 0 {
			userCount = parsed
		}
	}
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
	dbService, err := NewDatabaseService(dbPath)
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}
	defer dbService.Close()

	authController := NewAuthController(dbService)

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
