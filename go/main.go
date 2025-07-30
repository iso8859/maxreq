package main

import (
	"crypto/sha256"
	"database/sql"
	"fmt"
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
	_ "modernc.org/sqlite"
)

// LoginRequest represents the authentication request
type LoginRequest struct {
	Username       string `json:"Username" binding:"required"`
	HashedPassword string `json:"HashedPassword" binding:"required"`
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
	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, err
	}

	service := &DatabaseService{db: db}
	if err := service.InitializeDatabase(); err != nil {
		return nil, err
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
	count, err := ac.dbService.CreateTestUsers(10000)
	if err != nil {
		log.Printf("Error creating database: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": "An error occurred while creating the database",
		})
		return
	}

	message := fmt.Sprintf("Successfully created %d users in the database", count)
	c.String(http.StatusOK, message)
}

// HealthCheck handles health check endpoint
func HealthCheck(c *gin.Context) {
	c.String(http.StatusOK, "UserTokenApi Go server is running")
}

func main() {
	// Initialize database
	dbService, err := NewDatabaseService("users.db")
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}
	defer dbService.Close()

	// Initialize controller
	authController := NewAuthController(dbService)

	// Set Gin to release mode for better performance
	gin.SetMode(gin.ReleaseMode)

	// Create Gin router
	router := gin.New()
	
	// Add minimal middleware
	router.Use(gin.Recovery())

	// Setup routes
	router.GET("/health", HealthCheck)
	
	api := router.Group("/api")
	auth := api.Group("/auth")
	{
		auth.POST("/get-user-token", authController.GetUserToken)
		auth.GET("/create-db", authController.CreateDatabase)
	}

	// Start server
	port := "8082"
	fmt.Printf("Starting Go UserTokenApi server on http://localhost:%s\n", port)
	fmt.Println("Available endpoints:")
	fmt.Println("  GET /health - Health check")
	fmt.Println("  POST /api/auth/get-user-token - Authenticate user")
	fmt.Println("  GET /api/auth/create-db - Create test database with 10,000 users")

	log.Fatal(router.Run(":" + port))
}
