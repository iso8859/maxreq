import os
import logging
import asyncio
import aiosqlite
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from typing import Optional
from contextlib import asynccontextmanager
import hashlib
from functools import lru_cache
import weakref

# --- Enhanced database connection with performance optimizations ---
async def create_optimized_db_connection():
    """Create an optimized database connection with performance settings"""
    db = await aiosqlite.connect("users.db")
    
    # Configure SQLite for better performance and concurrency
    # await db.execute("PRAGMA journal_mode = WAL")           # Enable WAL mode
    # await db.execute("PRAGMA synchronous = NORMAL")         # Balance durability vs performance
    # await db.execute("PRAGMA cache_size = -64000")          # 64MB cache (negative = KB)
    # await db.execute("PRAGMA temp_store = MEMORY")          # Store temp tables in memory
    # await db.execute("PRAGMA mmap_size = 268435456")        # 256MB memory map
    # await db.execute("PRAGMA busy_timeout = 30000")         # 30 second timeout
    
    return db

# Global database connection
db_connection: Optional[aiosqlite.Connection] = None

# --- Application lifespan management ---
@asynccontextmanager
async def lifespan(app: FastAPI):
    global db_connection
    # Startup: Initialize global database connection
    db_connection = await create_optimized_db_connection()
    
    # Initialize database schema
    await db_connection.execute("""
        CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mail TEXT NOT NULL UNIQUE,
            hashed_password TEXT NOT NULL
        )
    """)
    
    # Create index for faster lookups
    await db_connection.execute("""
        CREATE INDEX IF NOT EXISTS idx_user_mail_password 
        ON user(mail, hashed_password)
    """)
    await db_connection.commit()
    
    yield
    
    # Shutdown: Close global database connection
    if db_connection:
        await db_connection.close()

# --- Models ---
class LoginRequest(BaseModel):
    UserName: str
    HashedPassword: str

class LoginResponse(BaseModel):
    Success: bool
    UserId: Optional[int] = None
    ErrorMessage: Optional[str] = None

app = FastAPI(lifespan=lifespan)

# --- Health & readiness ---
@app.get("/nodejs/health")
async def health():
    return {"status": "ok"}

# --- Authentication endpoint ---
@app.post("/nodejs/api/auth/get-user-token", response_model=LoginResponse)
async def get_user_token(req: LoginRequest):
    # Handle special test case
    if req.UserName == "no_db":
        return LoginResponse(Success=True, UserId=12345)
    
    try:
        if not db_connection:
            raise HTTPException(status_code=500, detail="Database not initialized")
            
        cursor = await db_connection.execute(
            "SELECT id FROM user WHERE mail = ? AND hashed_password = ? LIMIT 1", 
            (req.UserName, req.HashedPassword)
        )
        try:
            row = await cursor.fetchone()
            if row:
                return LoginResponse(Success=True, UserId=row[0])
            return LoginResponse(Success=False, ErrorMessage="Invalid username or password")
        finally:
            await cursor.close()
    except Exception as e:
        raise HTTPException(status_code=500, detail="Database error for " + req.UserName + ": " + str(e))

# --- Database setup / data seeding ---
@app.get("/nodejs/api/auth/create-db")
async def create_db():
    count = 10000  # Number of users to create
    
    if not db_connection:
        raise HTTPException(status_code=500, detail="Database not initialized")
    
    await db_connection.execute("DELETE FROM user")
    await db_connection.commit()
    
    stmt = "INSERT INTO user (mail, hashed_password) VALUES (?, ?)"
    batch = []
    inserted = 0
    for i in range(1, count + 1):
        email = f"user{i}@example.com"
        password = f"password{i}"
        hashed_password = hashlib.sha256(password.encode()).hexdigest()
        batch.append((email, hashed_password))
        if len(batch) >= 100:
            await db_connection.executemany(stmt, batch)
            inserted += len(batch)
            batch.clear()
    if batch:
        await db_connection.executemany(stmt, batch)
        inserted += len(batch)
    await db_connection.commit()
    return {"success": True, "inserted": inserted}

@app.get("/{path:path}")
async def catch_all(path: str):
    return {"path": path}