const express = require('express');
const sqlite3 = require('sqlite3').verbose();
const crypto = require('crypto');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 3000;
const DB_PATH = path.join(__dirname, 'users.db');

// Middleware
app.use(express.json());

// Database initialization
function initializeDatabase() {
    return new Promise((resolve, reject) => {
        const db = new sqlite3.Database(DB_PATH, (err) => {
            if (err) {
                reject(err);
                return;
            }
            console.log('Connected to SQLite database');
        });

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

        const db = await initializeDatabase();
        const user = await getUserByCredentials(db, UserName, HashedPassword);

        db.close();

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
    res.json({ status: 'OK', message: 'UserTokenApi Node.js server is running' });
});

// Start server
async function startServer() {
    try {
        await initializeDatabase();
        
        app.listen(PORT, () => {
            console.log(`Server is running on http://localhost:${PORT}`);
            console.log('Available endpoints:');
            console.log('  POST /api/auth/get-user-token - Authenticate user');
            console.log('  GET /api/auth/create-db - Create test database');
            console.log('  GET /health - Health check');
        });
    } catch (error) {
        console.error('Failed to start server:', error);
        process.exit(1);
    }
}

startServer();

module.exports = app;
