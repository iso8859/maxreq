<?php
// index.php - PHP UserToken API Implementation
// Requires PHP 8.0+ with SQLite extension

header('Content-Type: application/json');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');

// Handle preflight OPTIONS requests
if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(200);
    exit();
}

// Database connection
function getDatabase() {
    static $pdo = null;
    if ($pdo === null) {
        try {
            $pdo = new PDO('sqlite:users.db');
            $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
            $pdo->setAttribute(PDO::ATTR_DEFAULT_FETCH_MODE, PDO::FETCH_ASSOC);
            
            // Create table if it doesn't exist
            $pdo->exec("CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mail TEXT NOT NULL UNIQUE,
                hashed_password TEXT NOT NULL
            )");
        } catch (PDOException $e) {
            http_response_code(500);
            echo json_encode(['error' => 'Database connection failed: ' . $e->getMessage()]);
            exit();
        }
    }
    return $pdo;
}

// Routing
$requestUri = $_SERVER['REQUEST_URI'];
$requestMethod = $_SERVER['REQUEST_METHOD'];

// Remove query string
$path = parse_url($requestUri, PHP_URL_PATH);

// Route handling
switch (true) {
    case $path === '/health' && $requestMethod === 'GET':
        handleHealth();
        break;
        
    case $path === '/api/auth/get-user-token' && $requestMethod === 'POST':
        handleAuth();
        break;
        
    case preg_match('/^\/setup-database\/(\d+)$/', $path, $matches) && $requestMethod === 'POST':
        handleSetupDatabase((int)$matches[1]);
        break;
        
    default:
        http_response_code(404);
        echo json_encode(['error' => 'Not Found']);
        break;
}

function handleHealth() {
    echo json_encode('UserTokenApi PHP server is running');
}

function handleAuth() {
    try {
        // Get JSON input
        $input = file_get_contents('php://input');
        $data = json_decode($input, true);
        
        if (!$data) {
            http_response_code(400);
            echo json_encode([
                'success' => false,
                'userId' => null,
                'errorMessage' => 'Invalid JSON input'
            ]);
            return;
        }
        
        $username = $data['username'] ?? '';
        $hashedPassword = $data['hashedPassword'] ?? '';
        
        if (empty($username) || empty($hashedPassword)) {
            http_response_code(400);
            echo json_encode([
                'success' => false,
                'userId' => null,
                'errorMessage' => 'Username and hashedPassword are required'
            ]);
            return;
        }
        
        $pdo = getDatabase();
        
        // Find user by email
        $stmt = $pdo->prepare("SELECT id, hashed_password FROM user WHERE mail = ? LIMIT 1");
        $stmt->execute([$username]);
        $user = $stmt->fetch();
        
        if (!$user) {
            echo json_encode([
                'success' => false,
                'userId' => null,
                'errorMessage' => 'User not found'
            ]);
            return;
        }
        
        // Verify password
        if ($user['hashed_password'] === $hashedPassword) {
            echo json_encode([
                'success' => true,
                'userId' => (int)$user['id'],
                'errorMessage' => null
            ]);
        } else {
            echo json_encode([
                'success' => false,
                'userId' => null,
                'errorMessage' => 'Invalid password'
            ]);
        }
        
    } catch (Exception $e) {
        http_response_code(500);
        echo json_encode([
            'success' => false,
            'userId' => null,
            'errorMessage' => 'Internal server error: ' . $e->getMessage()
        ]);
    }
}

function handleSetupDatabase($count) {
    try {
        $pdo = getDatabase();
        
        // Clear existing data
        $pdo->exec("DELETE FROM user");
        
        // Prepare insert statement
        $stmt = $pdo->prepare("INSERT INTO user (mail, hashed_password) VALUES (?, ?)");
        
        // Hash for "password123"
        $hashedPassword = "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e";
        
        // Insert users in batches for better performance
        $pdo->beginTransaction();
        
        for ($i = 1; $i <= $count; $i++) {
            $email = "user{$i}@example.com";
            $stmt->execute([$email, $hashedPassword]);
            
            // Commit in batches of 1000
            if ($i % 1000 === 0) {
                $pdo->commit();
                if ($i < $count) {
                    $pdo->beginTransaction();
                }
            }
        }
        
        // Commit any remaining records
        if ($pdo->inTransaction()) {
            $pdo->commit();
        }
        
        echo json_encode("Successfully created {$count} users in the database");
        
    } catch (Exception $e) {
        if ($pdo->inTransaction()) {
            $pdo->rollBack();
        }
        http_response_code(500);
        echo json_encode(['error' => 'Database setup failed: ' . $e->getMessage()]);
    }
}
?>
