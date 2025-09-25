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

// log_line('INFO', 'Request start', array(
//     'method' => isset($_SERVER['REQUEST_METHOD']) ? $_SERVER['REQUEST_METHOD'] : '',
//     'uri' => isset($_SERVER['REQUEST_URI']) ? $_SERVER['REQUEST_URI'] : '',
//     'client' => isset($_SERVER['REMOTE_ADDR']) ? $_SERVER['REMOTE_ADDR'] : '',
// ));

// ====== Advanced Database Connection Pool with Prepared Statement Cache ======
class DatabasePool {
    private static $instance = null;
    private $connections = [];
    private $preparedStatements = [];
    private $maxConnections = 5;
    private $connectionInUse = [];
    
    private function __construct() {
        // Initialize connection pool
        for ($i = 0; $i < $this->maxConnections; $i++) {
            $this->connections[$i] = $this->createConnection();
            $this->connectionInUse[$i] = false;
        }
        // log_line('INFO', 'Database pool initialized', ['max_connections' => $this->maxConnections]);
    }
    
    public static function getInstance() {
        if (self::$instance === null) {
            self::$instance = new self();
        }
        return self::$instance;
    }
    
    private function createConnection() {
        try {
            $pdo = new PDO('sqlite:users.db', null, null, [
                PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
                PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
                PDO::ATTR_PERSISTENT => true,
                PDO::SQLITE_ATTR_OPEN_FLAGS => SQLITE3_OPEN_READWRITE | SQLITE3_OPEN_CREATE,
            ]);
            
            // SQLite optimizations
            $pdo->exec('PRAGMA journal_mode = WAL');
            $pdo->exec('PRAGMA synchronous = NORMAL');
            $pdo->exec('PRAGMA cache_size = 10000');
            $pdo->exec('PRAGMA temp_store = MEMORY');
            $pdo->exec('PRAGMA mmap_size = 268435456'); // 256MB
            
            return $pdo;
        } catch (PDOException $e) {
            // log_line('CRITICAL', 'Database connection failed', ['error' => $e->getMessage()]);
            throw $e;
        }
    }
    
    public function getConnection() {
        // Find available connection
        for ($i = 0; $i < $this->maxConnections; $i++) {
            if (!$this->connectionInUse[$i]) {
                $this->connectionInUse[$i] = true;
                return ['pdo' => $this->connections[$i], 'id' => $i];
            }
        }
        
        // All connections in use, create temporary connection
        // log_line('WARN', 'All connections in use, creating temporary connection');
        return ['pdo' => $this->createConnection(), 'id' => -1];
    }
    
    public function returnConnection($connectionInfo) {
        if ($connectionInfo['id'] >= 0) {
            $this->connectionInUse[$connectionInfo['id']] = false;
        }
        // Temporary connections (id = -1) will be garbage collected
    }
    
    public function getPreparedStatement($sql, $connectionInfo) {
        $connectionId = $connectionInfo['id'];
        $stmtKey = $connectionId . ':' . md5($sql);
        
        if (!isset($this->preparedStatements[$stmtKey])) {
            $this->preparedStatements[$stmtKey] = $connectionInfo['pdo']->prepare($sql);
            // log_line('DEBUG', 'Prepared statement cached', ['key' => $stmtKey]);
        }
        
        return $this->preparedStatements[$stmtKey];
    }
    
    public function clearStatementCache() {
        $this->preparedStatements = [];
    }
}

function getDatabase() {
    return DatabasePool::getInstance()->getConnection();
}

function returnDatabase($connectionInfo) {
    DatabasePool::getInstance()->returnConnection($connectionInfo);
}

function getPreparedStatement($sql, $connectionInfo) {
    return DatabasePool::getInstance()->getPreparedStatement($sql, $connectionInfo);
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
        echo json_encode(['Success'=>false,'UserId'=>null,'ErrorMessage'=>'Invalid JSON input']);
        return;
    }

    $username = $data['UserName'] ?? '';
    $hashedPassword = $data['HashedPassword'] ?? '';

    if ($username === '' || $hashedPassword === '') {
        http_response_code(400);
        // log_line('WARN', 'Missing credentials');
        echo json_encode(['Success'=>false,'UserId'=>null,'ErrorMessage'=>'Username and HashedPassword are required']);
        return;
    }

    // Special case for testing (similar to other implementations)
    if ($username === 'no_db') {
        echo json_encode(['Success'=>true,'UserId'=>1,'ErrorMessage'=>null]);
        return;
    }

    $connectionInfo = null;
    try {
        $connectionInfo = getDatabase();
        $stmt = getPreparedStatement("SELECT id, hashed_password FROM user WHERE mail = ? LIMIT 1", $connectionInfo);
        $stmt->execute([$username]);
        $user = $stmt->fetch();

        if (!$user) {
            // log_line('INFO', 'User not found', ['user'=>$username]);
            echo json_encode(['Success'=>false,'UserId'=>null,'ErrorMessage'=>'User not found']);
            return;
        }

        if (hash_equals($user['hashed_password'], $hashedPassword)) {
            // log_line('INFO', 'Auth success', ['user'=>$username]);
            echo json_encode(['Success'=>true,'UserId'=>(int)$user['id'],'ErrorMessage'=>null]);
        } else {
            // log_line('INFO', 'Auth failed (password mismatch)', ['user'=>$username]);
            echo json_encode(['Success'=>false,'UserId'=>null,'ErrorMessage'=>'Invalid password']);
        }
    } catch (Throwable $e) {
        // log_line('ERROR', 'Auth handler exception', ['error'=>$e->getMessage()]);
        http_response_code(500);
        echo json_encode(['Success'=>false,'UserId'=>null,'ErrorMessage'=>'Internal server error']);
    } finally {
        if ($connectionInfo) {
            returnDatabase($connectionInfo);
        }
    }
}

function handleSetupDatabase($count) {
    $t0 = microtime(true);
    // log_line('INFO', 'Seeding start', ['count'=>$count]);
    $connectionInfo = null;
    try {
        $connectionInfo = getDatabase();
        $pdo = $connectionInfo['pdo'];
        
        $pdo->exec("CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mail TEXT NOT NULL UNIQUE,
            hashed_password TEXT NOT NULL
        )");
        // Create index on mail for faster lookups
        $pdo->exec("CREATE INDEX IF NOT EXISTS idx_user_mail ON user(mail, hashed_password)");
        $pdo->exec("DELETE FROM user");
        
        $stmt = getPreparedStatement("INSERT INTO user (mail, hashed_password) VALUES (?, ?)", $connectionInfo);
        
        $pdo->beginTransaction();
        for ($i=1; $i <= $count; $i++) {
            $password = "password{$i}";
            $hashedPassword = hash('sha256', $password);
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
        echo json_encode(["Success"=>true, "Inserted"=>$count, "DurationMs"=>$elapsedMs]);
    } catch (Throwable $e) {
        if (isset($pdo) && $pdo->inTransaction()) $pdo->rollBack();
        // log_line('CRITICAL', 'Seeding failed', ['error'=>$e->getMessage()]);
        http_response_code(500);
        echo json_encode(['Success'=>false, 'ErrorMessage'=>'Database setup failed']);
    } finally {
        if ($connectionInfo) {
            returnDatabase($connectionInfo);
        }
    }
}

?>
