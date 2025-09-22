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

To run the server we start 16 instances behind a load balancer. Each instance listens on a different port.

Look at `ecosystem.config.js` for the configuration.

Look at start.bat / stop.bat for the command to start and stop the server using pm2.

Look at folder nginx for the nginx configuration to load balance between the instances.

##How to improve performance ?

Because we have 16 concurrent instances, we need to optimize SQLite for concurrency.

I don't know how to improve this right now. Any help is welcome.