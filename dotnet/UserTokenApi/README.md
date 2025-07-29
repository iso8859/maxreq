# UserTokenApi

A simple ASP.NET Core Web API for user authentication using SQLite database.

## Features

- **POST /api/auth/get-user-token**: Authenticate user with email and SHA256 hashed password
- **GET /api/auth/create-db**: Generate 10,000 test users in SQLite database

## Database Schema

The application uses a SQLite database (`users.db`) with the following table structure:

```sql
CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mail TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL
)
```

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

## Test Users

After running the create-db endpoint, you can test with these sample credentials:

- **Email**: `user1@example.com`
- **Password**: `password1`
- **SHA256 Hash**: `0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e`

## Running the Application

1. Navigate to the project directory:
   ```powershell
   Set-Location "e:\temp\maxreq\dotnet\UserTokenApi"
   ```

2. Run the application:
   ```powershell
   dotnet run
   ```

3. The API will be available at `https://localhost:7025` (or the port shown in the console)

4. First, call the create-db endpoint to populate the database with test users

5. Then test the authentication endpoint with the provided sample credentials

## Password Hashing

The API expects passwords to be SHA256 hashed on the client side. Here's how to generate a SHA256 hash:

**PowerShell:**
```powershell
$password = "password1"
$bytes = [System.Text.Encoding]::UTF8.GetBytes($password)
$hash = [System.Security.Cryptography.SHA256]::HashData($bytes)
$hashString = [System.BitConverter]::ToString($hash).Replace("-", "").ToLower()
Write-Output $hashString
```

**C#:**
```csharp
using System.Security.Cryptography;
using System.Text;

string password = "password1";
using SHA256 sha256Hash = SHA256.Create();
byte[] bytes = sha256Hash.ComputeHash(Encoding.UTF8.GetBytes(password));
string hash = Convert.ToHexString(bytes).ToLower();
```
