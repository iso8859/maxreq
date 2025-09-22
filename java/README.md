# User Token API - Java Spring Boot Implementation

A high-performance Java Spring Boot implementation of the User Token API for multi-language performance comparison.

## 🚀 Features

- **Spring Boot 3.2** - Modern Java web framework
- **SQLite Database** - Raw SQLite with JDBC driver
- **Async Operations** - CompletableFuture for non-blocking operations
- **JSON API** - RESTful endpoints with Jackson serialization
- **Connection Pooling** - Built-in connection management
- **Batch Processing** - Efficient bulk user creation
- **Maven 3.6+** - For dependency management

## 🚀 Running the Application

### Method 1: Maven
```bash
cd java
mvn spring-boot:run
```

### Method 2: Compile and Run
```bash
cd java
mvn clean compile
mvn exec:java -Dexec.mainClass="com.example.usertokenapi.UserTokenApiApplication"
```

### Method 3: Package and Run JAR
```bash
cd java
mvn clean package
java -jar target/user-token-api-0.0.1-SNAPSHOT.jar
```

The API will be available at: `http://localhost:6000`

