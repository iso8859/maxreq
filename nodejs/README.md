# UserTokenApi - Node.js

A simple Node.js Express API for user authentication using SQLite database with raw SQL queries.

## Features

- **POST /api/auth/get-user-token**: Authenticate user with email and SHA256 hashed password
- **GET /api/auth/create-db**: Generate 10,000 test users in SQLite database
- **GET /health**: Health check endpoint

## Database Schema

The application uses a SQLite database (`users.db`) with the following table structure:

```sql
CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mail TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL
)
```

## Installation

1. Navigate to the project directory:
   ```bash
   cd e:\temp\maxreq\nodejs
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Start the server:
   ```bash
   npm start
   ```

The API will be available at `http://localhost:3000`

## API Endpoints

### 1. Get User Token
**POST** `/api/auth/get-user-token`

Authenticates a user by email and SHA256 hashed password.

**Request Body:**
```json
{
  "username": "user1@example.com",
  "hashedPassword": "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
}
```

**Response:**
```json
{
  "success": true,
  "userId": 1,
  "errorMessage": null
}
```

### 2. Create Database
**GET** `/api/auth/create-db`

Creates 10,000 test users in the database. Users are created with emails like `user1@example.com` to `user10000@example.com` and passwords like `password1` to `password10000` (SHA256 hashed).

**Response:**
```
Successfully created 10000 users in the database
```

### 3. Health Check
**GET** `/health`

Simple health check endpoint.

**Response:**
```json
{
  "status": "OK",
  "message": "UserTokenApi Node.js server is running"
}
```

## Test Users

After running the create-db endpoint, you can test with these sample credentials:

- **Email**: `user1@example.com`
- **Password**: `password1`
- **SHA256 Hash**: `0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e`

## Password Hashing

The API expects passwords to be SHA256 hashed on the client side. Here's how to generate a SHA256 hash:

**Node.js:**
```javascript
const crypto = require('crypto');
const password = 'password1';
const hash = crypto.createHash('sha256').update(password).digest('hex');
console.log(hash); // 0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e
```

**PowerShell:**
```powershell
$password = "password1"
$bytes = [System.Text.Encoding]::UTF8.GetBytes($password)
$hash = [System.Security.Cryptography.SHA256]::HashData($bytes)
$hashString = [System.BitConverter]::ToString($hash).Replace("-", "").ToLower()
Write-Output $hashString
```

## Testing

You can test the API using curl, PowerShell, or any HTTP client:

**Create Database:**
```bash
curl -X GET "http://localhost:3000/api/auth/create-db"
```

**Authenticate User:**
```bash
curl -X POST "http://localhost:3000/api/auth/get-user-token" \
  -H "Content-Type: application/json" \
  -d '{"username":"user1@example.com","hashedPassword":"0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"}'
```

**PowerShell:**
```powershell
# Create database
Invoke-RestMethod -Uri "http://localhost:3000/api/auth/create-db" -Method Get

# Test authentication
$body = @{
    username = "user1@example.com"
    hashedPassword = "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:3000/api/auth/get-user-token" -Method Post -Body $body -ContentType "application/json"
```

## Dependencies

- **express**: Web framework for Node.js
- **sqlite3**: SQLite database driver for Node.js
- **crypto**: Built-in Node.js module for cryptographic functionality
