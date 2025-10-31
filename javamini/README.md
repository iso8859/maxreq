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

To setup Java on debian without docker
Setup https://sdkman.io/
intall maven
```bash
sdk install maven
```

To install graalvm
```bash
sdk install java 25-graal
```

To install Azul
```bash
sdk install java 21.0.4-zulu
```

To install temurin
```bash
sdk install java 25.0.1-tem
```

To build the project
```bash
mvn clean package
```

