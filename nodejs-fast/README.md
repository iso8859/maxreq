# Node.js Fast User Token API

A high-performance Node.js API built for speed by leveraging uWebSocket.js instead of the slower, widely-used Express framework.

We could use [ultimate-express](https://github.com/dimdenGD/ultimate-express) (which uses uWebSocket) instead, but as it tries to mimic the Express API, it loses a lot of CPU cycles.

If you think it is not fair to use uWebSocket.js (C/C++ module), the pure JS [restana](https://github.com/BackendStack21/restana) is also very fast and, most of the time, faster than `ultimate-express` without using uWebSocket.js. 

Is uWebSocket.js stable?
Yes. Bun.sh uses uWebSocket.js under the hood, and I've personally run it in production environments for nearly 8 years—serving a massive application across 200 servers at my previous company without issues.

Note: The overall HTTP server performance in Node.js 24 is approximately twice as fast compared to previous versions in cluster mode.


## Architecture

As we want to maximize the RAW performance, here are the best choices:
  - use only uWebSocket.js
  - better-sqlite3, with "offline" prepared statements
  - Node.js Cluster mode
  - my own simplified body parser, but I think the well-known bodyParser is fast enough (not tested). This part could be improved.
  - Even if the difference gets smaller over time, when you want high performance in JS, it is better to write plain JavaScript with callbacks (no async/await, TypeScript, etc.).


## Quick Start

Setup ultimate-express project
```bash
npm install ultimate-express
npm install better-sqlite3
```


```bash
# Must be NodeJS 24+
node index.js
```

- Use at least NodeJS 24.11.x as the HTTP server is 2x faster than NodeJS 22 and lower
- It is also better to [tweak Linux](https://www.digitalocean.com/community/tutorials/tuning-linux-performance-optimization#2-how-do-i-optimize-linux-tcp-performance-for-high-throughput-applications) to maximize TCP performance (impact on all languages: Rust, Node.js, ...)


## Endpoints

- **POST** `/nodejs/api/auth/get-user-token` - Authenticate user
  ```json
  {
    "UserName": "user1@example.com",
    "HashedPassword": "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
  }
  ```

- **GET** `/nodejs/api/auth/create-db` - Create 10,000 test users

- **GET** `/nodejshealth` - Health check (returns worker PID)


## Test


Create 10,000 users in the DB

```bash
curl http://localhost:8080/nodejs/api/auth/create-db
```

Query one user

```bash
curl -X POST "http://localhost:8080/nodejs/api/auth/get-user-token" \
  -H "Content-Type: application/json" \
  -d '{
    "UserName": "user100@example.com",
    "HashedPassword": "b3351ed9be23d5ad99cc73bdc1aed73913503f064534ead302d7485b72b072fe"
  }'
```

Status

```bash
curl http://localhost:8080/nodejshealth
```


## Results

I used [`rewrk`](https://github.com/lnx-search/rewrk) to test the endpoints. Unlike `wrk`, which uses HTTP/1 pipelining (a feature no major browser has supported for over a decade), `rewrk` sends standard HTTP/1.1 requests, producing numbers closer to real-world performance.

### HTTP/1.1

| Technology           | GET /health       | POST /api/auth/get-user-token |
|----------------------|------------------:|------------------------------:|
| NodeJs 24.11 Fast    |            TODO   |                        TODO   |
| Rust Actix           |            TODO   |                        TODO   |
| NodeJs 24.11 Express |            TODO   |                        TODO   |
| NodeJs 22.21 Express |            TODO   |                        TODO   |

### HTTP/2

| Technology           | GET /health       | POST /api/auth/get-user-token |
|----------------------|------------------:|------------------------------:|
| NodeJs 24.11 Fast    |            TODO   |                        TODO   |
| Rust Actix           |            TODO   |                        TODO   |
| NodeJs 24.11 Express |            TODO   |                        TODO   |
| NodeJs 22.21 Express |            TODO   |                        TODO   |

Install rewrk:
```bash
cargo install rewrk
```

### Test Machine Specifications

- OVH Public Cloud c3-16 instance
- 16 GB RAM
- 8 vCPUs @ 2.3 GHz
- 200 GB NVMe SSD
- Ubuntu 24.04

### Node.js:

```bash
# GET /nodejshealth (HTTP/1.1)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/nodejshealth

# GET /nodejshealth (HTTP/2)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/nodejshealth --http2

# Insert 10k users in DB
curl http://localhost:8080/nodejs/api/auth/create-db

# POST /nodejs/api/auth/get-user-token (HTTP/1.1)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/nodejs/api/auth/get-user-token \
  -m POST \
  -H "Content-Type: application/json" \
  -b '{"UserName":"user100@example.com","HashedPassword":"b3351ed9be23d5ad99cc73bdc1aed73913503f064534ead302d7485b72b072fe"}'

# POST /nodejs/api/auth/get-user-token (HTTP/2)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/nodejs/api/auth/get-user-token --http2 \
  -m POST \
  -H "Content-Type: application/json" \
  -b '{"UserName":"user100@example.com","HashedPassword":"b3351ed9be23d5ad99cc73bdc1aed73913503f064534ead302d7485b72b072fe"}'
```

### Rust Actix:

```bash
# GET /health (HTTP/1.1)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/health

# GET /health (HTTP/2)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/health --http2

# Insert 10k users in DB
curl http://localhost:8080/api/auth/create-db

# POST /api/auth/get-user-token (HTTP/1.1)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/api/auth/get-user-token \
  -m POST \
  -H "Content-Type: application/json" \
  -b '{"UserName":"user1@example.com","HashedPassword":"0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"}'

# POST /api/auth/get-user-token (HTTP/2)
rewrk -t 4 -c 400 -d 30s -h http://localhost:8080/api/auth/get-user-token --http2 \
  -m POST \
  -H "Content-Type: application/json" \
  -b '{"UserName":"user1@example.com","HashedPassword":"0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"}'
```

