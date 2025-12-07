const cluster = require('cluster');
const os = require('os');
const uWS = require('uWebSockets.js');
const sqlite3 = require('better-sqlite3');
const crypto = require('crypto');
const path = require('path');

const PORT = process.env.PORT || 8080;
const DB_PATH = path.join(__dirname, 'users.db');
const numCPUs = process.env.WEB_CONCURRENCY || os.cpus().length;

if (cluster.isMaster) {
    console.log(`Master process ${process.pid} is running`);
    console.log(`Starting ${numCPUs} worker processes...`);

    // Fork workers
    for (let i = 0; i < numCPUs; i++) {
        cluster.fork();
    }

    cluster.on('exit', (worker, code, signal) => {
        console.log(`Worker ${worker.process.pid} died. Starting a new worker...`);
        cluster.fork();
    });

} else {
    // Worker processes
    
    // Global database connection (connection pool per worker)
    // Initialize database pool
    const dbPool = initializeDatabase();
    const preparedStatements = prepareStatements();

    // Database initialization
    function initializeDatabase() {
        // Follow better-sqlite3 API (faster and same as Bun.js)
        const db = new sqlite3(DB_PATH, {
            // verbose : console.log,
        });
        console.log(`Worker ${process.pid}: Connected to SQLite database`);
        // Enable WAL mode:
        // - significantly faster.
        // - provides more concurrency as readers do not block writers and a writer does not block readers. Reading and writing can proceed concurrently.
        db.pragma('journal_mode = WAL');
        db.pragma('synchronous = normal');
        // Create table if it doesn't exist. TODO we should try catch in a real world scenario.
        db.exec(`CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mail TEXT NOT NULL UNIQUE,
            hashed_password TEXT NOT NULL
        )`);
        return db;
    }

    function prepareStatements() {
        const statements = {};
        // Prepare statement for getting user by email and password
        statements.getUserByCredentials = (() => {
            const _plan = dbPool.prepare('SELECT id, mail, hashed_password FROM user WHERE mail = ? AND hashed_password = ?');
            return (email, hashedPassword) => _plan.get([email, hashedPassword]);
        })();

        // Delete all users
        statements.deleteAllUsers = (() => {
            const _plan = dbPool.prepare('DELETE FROM user');
            return () => _plan.run();
        })();

        // Count all users
        statements.countUsers = (() => {
            const _plan = dbPool.prepare('SELECT COUNT(*) as count FROM user');
            return () => _plan.get();
        })();

        // Prepare statement for inserting bulk users (arrays of [email, hashedPassword])
        statements.insertBulkUsers = (() => {
            const _plan = dbPool.prepare('INSERT INTO user (mail, hashed_password) VALUES (?, ?)');
            return (users) => {
                const transaction = dbPool.transaction(() => {
                    for (const user of users) {
                        _plan.run(user);
                    }
                });
                return transaction();
            };
        })();

        return statements;
    }

    // Helper function to hash password with SHA256
    function hashPassword(password) {
        return crypto.createHash('sha256').update(password).digest('hex');
    }

    // Helper function to create test users
    function createTestUsers(count = 10000) {
        try {
            preparedStatements.deleteAllUsers();
            const users = [];
            for (let i = 1; i <= count; i++) {
                const email = `user${i}@example.com`;
                const password = `password${i}`;
                const hashedPassword = hashPassword(password);
                users.push([email, hashedPassword]);
            }
            preparedStatements.insertBulkUsers(users);
            const nbUsersInserted = preparedStatements.countUsers().count;
            console.log(nbUsersInserted);
            return nbUsersInserted;
        }
        catch (error) {
            console.error('Error creating test users:', error);
            return 0;
        }
    }

    // Helper function to parse JSON body
    function parseJSON(res, callback) {
        let buffer;
        res.onData((chunk, isLast) => {
            const cur = Buffer.from(chunk);
            if (buffer) {
                buffer = Buffer.concat([buffer, cur]);
            } else {
                buffer = cur;
            }
            if (isLast) {
                try {
                    const json = JSON.parse(buffer.toString());
                    callback(json);
                } catch (e) {
                    res.writeStatus('400 Bad Request');
                    res.end(JSON.stringify({ error: 'Invalid JSON' }));
                }
            }
        });

        res.onAborted(() => {
            // Request was aborted
        });
    }

    // Helper function to send JSON response
    function sendJSON(res, data, status = '200 OK') {
        res.writeStatus(status);
        res.writeHeader('Content-Type', 'application/json');
        res.end(JSON.stringify(data));
    }

    // Helper function to send text response
    function sendText(res, text, status = '200 OK') {
        res.writeStatus(status);
        res.writeHeader('Content-Type', 'text/plain');
        res.end(text);
    }

    // Create uWebSockets.js app
    uWS.App({})
        .get('/', (res, req) => {
            sendJSON(res, {
                status: 'OK',
                message: 'Node.js Fast User Token API',
                worker: process.pid,
                endpoints: [
                    'GET /nodejshealth',
                    'GET /nodejs/api/auth/create-db',
                    'POST /nodejs/api/auth/get-user-token'
                ]
            });
        })
        .get('/nodejshealth', (res, req) => {
            sendJSON(res, {
                status: 'OK',
                message: 'UserTokenApi Node.js server is running',
                worker: process.pid
            });
        })
        .get('/nodejs/api/auth/create-db', (res, req) => {
            try {
                const count = createTestUsers(10000);
                sendText(res, `Successfully created ${count} users in the database`);
            } catch (error) {
                console.error('Error creating database:', error);
                sendText(res, 'An error occurred while creating the database', '500 Internal Server Error');
            }
        })
        .post('/nodejs/api/auth/get-user-token', (res, req) => {
            parseJSON(res, (body) => {
                try {
                    const { UserName, HashedPassword } = body;

                    if (UserName === "no_db") {
                        return sendJSON(res, {
                            success: true,
                            userId: 1,
                            errorMessage: null
                        });
                    }

                    const user = preparedStatements.getUserByCredentials(UserName, HashedPassword);

                    if (user) {
                        return sendJSON(res, {
                            success: true,
                            userId: user.id,
                            errorMessage: null
                        });
                    } else {
                        return sendJSON(res, {
                            success: false,
                            userId: null,
                            errorMessage: 'Invalid username or password'
                        });
                    }
                } catch (error) {
                    console.error('Error during authentication:', error);
                    return sendJSON(res, {
                        success: false,
                        userId: null,
                        errorMessage: 'An error occurred during authentication'
                    });
                }
            });
        })
        .listen(PORT, (token) => {
            if (token) {
                console.log(`Worker ${process.pid} is running on http://localhost:${PORT}`);
            } else {
                console.error(`Worker ${process.pid} failed to start on port ${PORT}`);
                process.exit(1);
            }
        });

    // Graceful shutdown
    process.on('SIGINT', () => {
        console.log(`Worker ${process.pid} shutting down gracefully...`);
        dbPool?.close?.();
        process.exit(0);
    });
}

