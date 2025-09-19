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

# --- Configuration via environment variables ---
DB_PATH = os.getenv("DB_PATH", "users.db")
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO").upper()
DB_INIT_RETRIES = int(os.getenv("DB_INIT_RETRIES", "5"))
DB_INIT_DELAY_SEC = float(os.getenv("DB_INIT_DELAY_SEC", "1"))
BATCH_SIZE = int(os.getenv("BATCH_SIZE", "1000"))

# --- Logging setup ---
logging.basicConfig(
    level=LOG_LEVEL,
    format='%(asctime)s | %(levelname)s | pid=%(process)d | %(message)s'
)
logger = logging.getLogger("usertoken-python")

# --- Models ---
class LoginRequest(BaseModel):
    username: str
    hashedPassword: str

class LoginResponse(BaseModel):
    success: bool
    userId: Optional[int] = None
    errorMessage: Optional[str] = None

# --- Database initialization with retry ---
async def initialize_database():
    for attempt in range(1, DB_INIT_RETRIES + 1):
        try:
            async with aiosqlite.connect(DB_PATH) as db:
                await db.execute("""
                    CREATE TABLE IF NOT EXISTS user (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        mail TEXT NOT NULL UNIQUE,
                        hashed_password TEXT NOT NULL
                    )
                """)
                await db.commit()
            logger.info("Database initialized (attempt %d)", attempt)
            return
        except Exception as e:
            logger.warning("Database init attempt %d/%d failed: %s", attempt, DB_INIT_RETRIES, e)
            if attempt == DB_INIT_RETRIES:
                logger.critical("Exhausted database initialization attempts; aborting startup.")
                raise
            await asyncio.sleep(DB_INIT_DELAY_SEC)

# --- Lifespan management ---
@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Startup: initializing resources (DB path=%s)", DB_PATH)
    await initialize_database()
    yield
    logger.info("Shutdown: resources released")

app = FastAPI(lifespan=lifespan)

# --- Global exception handler ---
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    logger.error("Unhandled error on %s %s: %s", request.method, request.url, exc, exc_info=True)
    return JSONResponse(status_code=500, content={"success": False, "errorMessage": "Internal server error"})

# --- Health & readiness ---
@app.get("/health")
async def health():
    return {"status": "ok"}

@app.get("/ready")
async def readiness():
    # Could be extended with deeper checks
    try:
        async with aiosqlite.connect(DB_PATH) as db:
            await db.execute("SELECT 1")
        return {"ready": True}
    except Exception as e:
        logger.error("Readiness check failed: %s", e)
        return JSONResponse(status_code=500, content={"ready": False})

# --- Authentication endpoint ---
@app.post("/api/auth/get-user-token", response_model=LoginResponse)
async def get_user_token(req: LoginRequest):
    try:
        async with aiosqlite.connect(DB_PATH) as db:
            async with db.execute(
                "SELECT id FROM user WHERE mail = ? AND hashed_password = ?", (req.username, req.hashedPassword)
            ) as cursor:
                row = await cursor.fetchone()
                if row:
                    return LoginResponse(success=True, userId=row[0])
                return LoginResponse(success=False, errorMessage="Invalid username or password")
    except Exception as e:
        logger.error("Auth error for user %s: %s", req.username, e, exc_info=True)
        raise HTTPException(status_code=500, detail="Database error")

# --- Database setup / data seeding ---
@app.post("/setup-database/{count}")
async def setup_database(count: int):
    if count < 0:
        raise HTTPException(status_code=400, detail="Count must be non-negative")
    logger.info("Seeding database with %d users (batch=%d)", count, BATCH_SIZE)
    try:
        async with aiosqlite.connect(DB_PATH) as db:
            await db.execute("DELETE FROM user")
            await db.commit()
            stmt = "INSERT INTO user (mail, hashed_password) VALUES (?, ?)"
            batch = []
            inserted = 0
            for i in range(1, count + 1):
                email = f"user{i}@example.com"
                password = f"password{i}"
                hashed_password = hashlib.sha256(password.encode()).hexdigest()
                batch.append((email, hashed_password))
                if len(batch) >= BATCH_SIZE:
                    await db.executemany(stmt, batch)
                    inserted += len(batch)
                    batch.clear()
            if batch:
                await db.executemany(stmt, batch)
                inserted += len(batch)
            await db.commit()
        logger.info("Seed complete: %d users inserted", inserted)
        return {"success": True, "inserted": inserted}
    except Exception as e:
        logger.error("Seeding failed: %s", e, exc_info=True)
        raise HTTPException(status_code=500, detail="Database seeding error")
