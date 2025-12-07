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

I used `wrk` to test the endpoints.

| Technology           | GET /health       | POST /api/auth/get-user-token |
|----------------------|------------------:|------------------------------:|
| NodeJs 24.11 Fast    |        386533 RPS |                    232290 RPS |
| Rust Actix           |        382089 RPS |    (with errors)   298869 RPS |
| NodeJs 24.11 Express |         96674 RPS |                     30934 RPS |
| NodeJs 22.21 Express |         46977 RPS |                     21670 RPS |


### Test Machine Specifications

- OVH Public Cloud c3-16 instance
- 16 GB RAM
- 8 vCPUs @ 2.3 GHz
- 200 GB NVMe SSD
- Ubuntu 24.04

### Node.js:

```bash
# GET /nodejshealth
wrk -t4 -c400 -d30s http://localhost:8080/nodejshealth
Running 30s test @ http://localhost:8080/nodejshealth
  4 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.38ms    1.50ms  15.15ms   82.85%
    Req/Sec    97.16k     6.36k  116.37k    85.42%
  11599261 requests in 30.01s, 2.21GB read
Requests/sec: 386533.68
Transfer/sec:     75.57MB

# Insert 10k users in DB
curl http://localhost:8080/nodejs/api/auth/create-db

# POST /nodejs/api/auth/get-user-token
wrk -t4 -c400 -d30s -s post.lua http://localhost:8080/nodejs/api/auth/get-user-token
Running 30s test @ http://localhost:8080/nodejs/api/auth/get-user-token
  4 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.94ms    1.76ms  18.30ms   82.93%
    Req/Sec    58.41k    13.38k  102.68k    77.67%
  6983530 requests in 30.06s, 1.31GB read
Requests/sec: 232290.11
Transfer/sec:     44.75MB
```

### Rust Actix:

WARNING: I don't know why I have so many errors with Rust for POST requests.
So the result may not be correct. TODO fix it.

```bash
# GET /health
wrk -t4 -c400 -d30s http://localhost:8080/health
Running 30s test @ http://localhost:8080/health
  4 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.54ms    2.06ms  62.95ms   90.20%
    Req/Sec    96.06k     5.27k  108.00k    71.00%
  11467161 requests in 30.01s, 2.43GB read
Requests/sec: 382089.32
Transfer/sec:     83.08MB

# Insert 10k users in DB
curl http://localhost:8080/api/auth/create-db

# POST /api/auth/get-user-token
wrk -t4 -c400 -d30s -s post.lua   http://localhost:8080/api/auth/get-user-token
Running 30s test @ http://localhost:8080/api/auth/get-user-token
  4 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.68ms    1.74ms  18.86ms   83.11%
    Req/Sec    75.18k    11.61k  102.68k    66.00%
  8980448 requests in 30.05s, 3.02GB read
  Non-2xx or 3xx responses: 8980448
Requests/sec: 298869.25
Transfer/sec:    102.89MB

```

