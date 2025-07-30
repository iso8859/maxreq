# Python UserToken API

A high-performance Python implementation of the UserToken API using:
- **FastAPI** (async, modern web framework)
- **SQLite** (aiosqlite for async DB access)
- **Pydantic** for data validation

## Requirements
- Python 3.10+
- Install dependencies:
  ```bash
  pip install -r requirements.txt
  ```

## Running the API
```bash
cd python
uvicorn main:app --host 0.0.0.0 --port 7000 --workers 16 --log-level critical
# API runs at http://localhost:7000
```

## API Endpoints
- `GET /health` — Health check
- `POST /api/auth/get-user-token` — User authentication
- `POST /setup-database/{count}` — Bulk user creation

## Example Usage
```bash
# Health check
curl http://localhost:7000/health

# Setup test data
curl -X POST http://localhost:7000/setup-database/1000

# Test authentication
curl -X POST http://localhost:7000/api/auth/get-user-token \
  -H "Content-Type: application/json" \
  -d '{"username":"user1@example.com","hashedPassword":"0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"}'
```

## Performance Notes
- Use `--workers` to scale (for true concurrency, use multiple processes)
- For best results, run with `uvicorn` and a single worker for apples-to-apples comparison
