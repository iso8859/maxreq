# Java Minimal API (javamini)

A minimal Java HTTP server mirroring the Go minimal API style for the UserTokenApi benchmark.

## Features
- com.sun.net.httpserver.HttpServer (JDK built-in)
- SQLite via xerial sqlite-jdbc
- JSON via Jackson
- Endpoints: /health, /ready, /api/auth/get-user-token (POST), /api/auth/create-db (GET)
- Env config: PORT (default 6000), DB_PATH (users.db), READ_ONLY_DB/READ_ONLY, SEED_USER_COUNT

## Run
```bash
cd javamini
mvn clean package
java -jar target/javamini-1.0.0-shaded.jar
```

