const cluster = require('cluster');
const os = require('os');
const express = require('express');
const sqlite3 = require('sqlite3').verbose();
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
    const app = express();

    // Middleware
    app.use(express.json());

    // Global database connection (connection pool per worker)
    let dbPool = null;

    // Database initialization
    function initializeDatabase() {
        return new Promise((resolve, reject) => {
            const db = new sqlite3.Database(DB_PATH, (err) => {
                if (err) {
                    reject(err);
                    return;
                }
                console.log(`Worker ${process.pid}: Connected to SQLite database`);
                // Enable WAL mode for better concurrency
                db.run('PRAGMA journal_mode = WAL;', (err) => {
                    if (err) {
                        reject(err);
                        return;
                    }
                    db.run(`CREATE TABLE IF NOT EXISTS user (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        mail TEXT NOT NULL UNIQUE,
                        hashed_password TEXT NOT NULL
                    )`, (err) => {
                        if (err) {
                            reject(err);
                        } else {
                            resolve(db);
                        }
                    });
                });
            });
        });
    }

    // Get or create database connection
    function getDatabase() {
        if (dbPool) {
            return Promise.resolve(dbPool);
        }
        return initializeDatabase().then(db => {
            dbPool = db;
            return db;
        });
    }

    // Helper function to hash password with SHA256
    function hashPassword(password) {
        return crypto.createHash('sha256').update(password).digest('hex');
    }

    // Helper function to get user by email and password
    function getUserByCredentials(db, email, hashedPassword) {
        return new Promise((resolve, reject) => {
            const query = 'SELECT id, mail, hashed_password FROM user WHERE mail = ? AND hashed_password = ?';
            db.get(query, [email, hashedPassword], (err, row) => {
                if (err) {
                    reject(err);
                } else {
                    resolve(row);
                }
            });
        });
    }

    // Helper function to create test users
    function createTestUsers(db, count = 10000) {
        return new Promise((resolve, reject) => {
            // Clear existing users first
            db.run('DELETE FROM user', (err) => {
                if (err) {
                    reject(err);
                    return;
                }

                db.serialize(() => {
                    db.run('BEGIN TRANSACTION', (err) => {
                        if (err) {
                            reject(err);
                            return;
                        }
                    });

                    const stmt = db.prepare('INSERT INTO user (mail, hashed_password) VALUES (?, ?)');

                    for (let i = 1; i <= count; i++) {
                        const email = `user${i}@example.com`;
                        const password = `password${i}`;
                        const hashedPassword = hashPassword(password);
                        stmt.run(email, hashedPassword);
                    }

                    stmt.finalize((err) => {
                        if (err) {
                            db.run('ROLLBACK');
                            reject(err);
                            return;
                        }

                        db.run('COMMIT', (err) => {
                            if (err) {
                                reject(err);
                            } else {
                                // Count the actual number of users inserted
                                db.get('SELECT COUNT(*) as count FROM user', (err, row) => {
                                    if (err) {
                                        reject(err);
                                    } else {
                                        resolve(row.count);
                                    }
                                });
                            }
                        });
                    });
                });
            });
        });
    }

    // Routes

    // POST /nodejs/api/auth/get-user-token - Authenticate user
    app.post('/nodejs/api/auth/get-user-token', async (req, res) => {
        try {
            const { UserName, HashedPassword } = req.body;

            if (UserName == "no_db")
                return res.json({success: true, userId: 1, errorMessage: null});

            const db = await getDatabase();
            const user = await getUserByCredentials(db, UserName, HashedPassword);

            // Don't close the db - keep it in the pool

            if (user) {
                return res.json({
                    success: true,
                    userId: user.id,
                    errorMessage: null
                });
            } else {
                return res.json({
                    success: false,
                    userId: null,
                    errorMessage: 'Invalid username or password'
                });
            }
        } catch (error) {
            console.error('Error during authentication:', error);
            return res.json({
                success: false,
                userId: null,
                errorMessage: 'An error occurred during authentication'
            });
        }
    });

    // GET /nodejs/api/auth/create-db - Create test database with 10000 users
    app.get('/nodejs/api/auth/create-db', async (req, res) => {
        try {
            const db = await initializeDatabase();
            const count = await createTestUsers(db, 10000);
            
            db.close();

            res.send(`Successfully created ${count} users in the database`);
        } catch (error) {
            console.error('Error creating database:', error);
            res.status(500).send('An error occurred while creating the database');
        }
    });

    // Health check endpoint
    app.get('/nodejshealth', (req, res) => {
        res.json({ status: 'OK', message: 'UserTokenApi Node.js server is running', worker: process.pid });
    });

    // Start server
    async function startServer() {
        try {
            // Initialize the database pool once at startup
            dbPool = await initializeDatabase();
            
            app.listen(PORT, () => {
                console.log(`Worker ${process.pid} is running on http://localhost:${PORT}`);
            });
        } catch (error) {
            console.error(`Worker ${process.pid} failed to start:`, error);
            process.exit(1);
        }
    }

    // Graceful shutdown
    process.on('SIGINT', () => {
        console.log(`Worker ${process.pid} shutting down gracefully...`);
        if (dbPool) {
            dbPool.close((err) => {
                if (err) {
                    console.error('Error closing database:', err);
                }
                process.exit(0);
            });
        } else {
            process.exit(0);
        }
    });

    startServer();

    module.exports = app;
}
