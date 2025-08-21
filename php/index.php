<?php
// index.php - PHP UserToken API Implementation with diagnostic tracing
// Requires PHP 8.0+ with SQLite extension

// ====== Runtime Diagnostics & Configuration ======
$START_TIME = microtime(true);
$REQ_ID = bin2hex(random_bytes(4));
$LOG_FILE = getenv('PHP_API_LOG') ?: __DIR__ . '/php_api.log';
$LOG_LEVEL = strtoupper(getenv('PHP_API_LOG_LEVEL') ?: 'ERROR');
$TRACE_MEMORY = filter_var(getenv('PHP_API_TRACE_MEMORY') ?: '1', FILTER_VALIDATE_BOOL);

function log_line(string $level, string $message, array $ctx = []): void {
    //return;
    global $LOG_FILE, $REQ_ID, $LOG_LEVEL;
    static $levelOrder = [ 'DEBUG'=>0, 'INFO'=>1, 'WARN'=>2, 'ERROR'=>3, 'CRITICAL'=>4 ];
    $cfgLevel = $levelOrder[$LOG_LEVEL] ?? 1;
    $msgLevel = $levelOrder[$level] ?? 1;
    if ($msgLevel < $cfgLevel) return;
    $ts = date('c');
    $ctxStr = '';
    if (!empty($ctx)) {
        $safe = [];
        foreach ($ctx as $k=>$v) { $safe[$k] = is_scalar($v)? $v : json_encode($v); }
        $ctxStr = ' ' . json_encode($safe, JSON_UNESCAPED_SLASHES);
    }
    @file_put_contents($LOG_FILE, "$ts [$level] rid=$REQ_ID $message$ctxStr\n", FILE_APPEND);
}

// Error & exception handling
set_error_handler(function($severity, $message, $file, $line) {
    if (!(error_reporting() & $severity)) { return false; }
    // log_line('ERROR', 'PHP Error', ['severity'=>$severity, 'msg'=>$message, 'file'=>$file, 'line'=>$line]);
    return false; // Let PHP continue normal handling
});

set_exception_handler(function(Throwable $e){
    // log_line('CRITICAL', 'Uncaught Exception', ['type'=>get_class($e), 'msg'=>$e->getMessage(), 'file'=>$e->getFile(), 'line'=>$e->getLine(), 'trace'=>$e->getTraceAsString()]);
    http_response_code(500);
    echo json_encode(['success'=>false, 'errorMessage'=>'Internal server error']);
});

register_shutdown_function(function() use ($START_TIME, $TRACE_MEMORY){
    $err = error_get_last();
    if ($err) {
        // log_line('CRITICAL', 'Shutdown with fatal error', $err);
    } else {
        // log_line('INFO', 'Shutdown normal');
    }
    if ($TRACE_MEMORY) {
        // log_line('DEBUG', 'Memory usage summary', [
        //    'mem_current'=>memory_get_usage(),
        //    'mem_peak'=>memory_get_peak_usage(),
        //    'duration_ms'=>round((microtime(true)-$START_TIME)*1000,2)
        // ]);
    }
});

// Basic runtime config
ini_set('display_errors', '0');
ini_set('log_errors', '1');
ini_set('error_log', $LOG_FILE);
header('X-Request-Id: ' . $REQ_ID);

// CORS & common headers
header('Content-Type: application/json');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type, X-Request-Id');

if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(200);
    // log_line('DEBUG', 'Handled CORS preflight');
    exit();
}

// log_line('INFO', 'Request start', [
    'method'=>$_SERVER['REQUEST_METHOD'] ?? '',
    'uri'=>$_SERVER['REQUEST_URI'] ?? '',
    'client'=>$_SERVER['REMOTE_ADDR'] ?? '',
]);

// ====== Database Connection ======
function getDatabase() {
    static $pdo = null;
    if ($pdo === null) {
        try {
            $pdo = new PDO('sqlite:users.db');
            $pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);
            $pdo->setAttribute(PDO::ATTR_DEFAULT_FETCH_MODE, PDO::FETCH_ASSOC);
            $pdo->exec("PRAGMA journal_mode = WAL");
            $pdo->exec("PRAGMA synchronous = NORMAL");
            $pdo->exec("CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mail TEXT NOT NULL UNIQUE,
                hashed_password TEXT NOT NULL
            )");
            // log_line('INFO', 'Database initialized');
        } catch (PDOException $e) {
            // log_line('CRITICAL', 'Database connection failed', ['error'=>$e->getMessage()]);
            http_response_code(500);
            echo json_encode(['error' => 'Database connection failed']);
            exit();
        }
    }
    return $pdo;
}

// ====== Routing ======
$requestUri = $_SERVER['REQUEST_URI'] ?? '/';
$requestMethod = $_SERVER['REQUEST_METHOD'] ?? 'GET';
$path = parse_url($requestUri, PHP_URL_PATH);

try {
    switch (true) {
        case $path === '/php/api/auth/health' && $requestMethod === 'GET':
            handleHealth();
            break;
        case $path === '/php/api/auth/get-user-token' && $requestMethod === 'POST':
            handleAuth();
            break;
        case $path === '/php/api/auth/create-db' && $requestMethod === 'GET':
            handleSetupDatabase(10000);
            break;
        case preg_match('/^\/php\/setup-database\/(\d+)$/', $path, $matches) && $requestMethod === 'POST':
            handleSetupDatabase((int)$matches[1]);
            break;
        default:
            http_response_code(404);
            echo json_encode(['error' => 'Not Found', 'path'=>$path]);
            // log_line('WARN', 'Route not found', ['path'=>$path]);
            break;
    }
} catch (Throwable $t) {
    // log_line('CRITICAL', 'Unhandled routing exception', ['msg'=>$t->getMessage()]);
    http_response_code(500);
    echo json_encode(['error'=>'Internal server error']);
}

// ====== Handlers ======
function handleHealth() {
    // log_line('DEBUG', 'Health check');
    echo json_encode('UserTokenApi PHP server is running');
}

function handleAuth() {
    $raw = file_get_contents('php://input');
    // log_line('DEBUG', 'Auth request raw body', ['len'=>strlen($raw)]);
    try {
        $data = json_decode($raw, true, 512, JSON_THROW_ON_ERROR);
    } catch (Throwable $e) {
        http_response_code(400);
        // log_line('WARN', 'Invalid JSON', ['error'=>$e->getMessage()]);
        echo json_encode(['success'=>false,'userId'=>null,'errorMessage'=>'Invalid JSON input']);
        return;
    }

    $username = $data['username'] ?? '';
    $hashedPassword = $data['hashedPassword'] ?? '';

    if ($username === '' || $hashedPassword === '') {
        http_response_code(400);
        // log_line('WARN', 'Missing credentials');
        echo json_encode(['success'=>false,'userId'=>null,'errorMessage'=>'Username and hashedPassword are required']);
        return;
    }

    try {
        $pdo = getDatabase();
        $stmt = $pdo->prepare("SELECT id, hashed_password FROM user WHERE mail = ? LIMIT 1");
        $stmt->execute([$username]);
        $user = $stmt->fetch();

        if (!$user) {
            // log_line('INFO', 'User not found', ['user'=>$username]);
            echo json_encode(['success'=>false,'userId'=>null,'errorMessage'=>'User not found']);
            return;
        }

        if (hash_equals($user['hashed_password'], $hashedPassword)) {
            // log_line('INFO', 'Auth success', ['user'=>$username]);
            echo json_encode(['success'=>true,'userId'=>(int)$user['id'],'errorMessage'=>null]);
        } else {
            // log_line('INFO', 'Auth failed (password mismatch)', ['user'=>$username]);
            echo json_encode(['success'=>false,'userId'=>null,'errorMessage'=>'Invalid password']);
        }
    } catch (Throwable $e) {
        // log_line('ERROR', 'Auth handler exception', ['error'=>$e->getMessage()]);
        http_response_code(500);
        echo json_encode(['success'=>false,'userId'=>null,'errorMessage'=>'Internal server error']);
    }
}

function handleSetupDatabase($count) {
    $t0 = microtime(true);
    // log_line('INFO', 'Seeding start', ['count'=>$count]);
    try {
        $pdo = getDatabase();
        $pdo->exec("DELETE FROM user");
        $stmt = $pdo->prepare("INSERT INTO user (mail, hashed_password) VALUES (?, ?)");
        $hashedPassword = "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e";
        $pdo->beginTransaction();
        for ($i=1; $i <= $count; $i++) {
            $stmt->execute(["user{$i}@example.com", $hashedPassword]);
            if ($i % 1000 === 0) {
                $pdo->commit();
                // log_line('DEBUG', 'Batch committed', ['inserted'=>$i]);
                if ($i < $count) $pdo->beginTransaction();
            }
        }
        if ($pdo->inTransaction()) $pdo->commit();
        $elapsedMs = round((microtime(true)-$t0)*1000,2);
        // log_line('INFO', 'Seeding complete', ['inserted'=>$count,'ms'=>$elapsedMs]);
        echo json_encode(["success"=>true, "inserted"=>$count]);
    } catch (Throwable $e) {
        if (isset($pdo) && $pdo->inTransaction()) $pdo->rollBack();
        // log_line('CRITICAL', 'Seeding failed', ['error'=>$e->getMessage()]);
        http_response_code(500);
        echo json_encode(['error'=>'Database setup failed']);
    }
}

?>
