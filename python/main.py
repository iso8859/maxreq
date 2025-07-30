import aiosqlite
from fastapi import FastAPI, HTTPException, Request
from pydantic import BaseModel
from typing import Optional
import hashlib

app = FastAPI()
DB_PATH = "users.db"

class LoginRequest(BaseModel):
    Username: str
    HashedPassword: str

class LoginResponse(BaseModel):
    success: bool
    userId: Optional[int] = None
    errorMessage: Optional[str] = None

@app.on_event("startup")
async def startup():
    async with aiosqlite.connect(DB_PATH) as db:
        await db.execute("""
            CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mail TEXT NOT NULL UNIQUE,
                hashed_password TEXT NOT NULL
            )
        """)
        await db.commit()

@app.get("/health")
async def health():
    return "UserTokenApi Python server is running"

@app.post("/api/auth/get-user-token", response_model=LoginResponse)
async def get_user_token(req: LoginRequest):
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("SELECT id FROM user WHERE mail = ? AND hashed_password = ?", (req.Username, req.HashedPassword)) as cursor:
            row = await cursor.fetchone()
            if row:
                user_id = row[0]
                return LoginResponse(success=True, userId=user_id, errorMessage=None)
            else:
                return LoginResponse(success=False, userId=None, errorMessage="Invalid username or password")

@app.post("/setup-database/{count}")
async def setup_database(count: int):
    async with aiosqlite.connect(DB_PATH) as db:
        await db.execute("DELETE FROM user")
        await db.commit()
        
        stmt = "INSERT INTO user (mail, hashed_password) VALUES (?, ?)"
        hashed_password = "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"  # password123
        
        # Insert in batches for better performance
        batch_size = 1000
        for batch_start in range(1, count + 1, batch_size):
            batch_end = min(batch_start + batch_size - 1, count)
            
            async with db.execute("BEGIN TRANSACTION"):
                for i in range(batch_start, batch_end + 1):
                    await db.execute(stmt, (f"user{i}@example.com", hashed_password))
                await db.execute("COMMIT")
                
    return f"Successfully created {count} users in the database"
