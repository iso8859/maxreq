# User Token API - Java Spring Boot Implementation

A high-performance Java Spring Boot implementation of the User Token API for multi-language performance comparison.

## 🚀 Features

- **Spring Boot 3.2** - Modern Java web framework
- **SQLite Database** - Raw SQLite with JDBC driver
- **Async Operations** - CompletableFuture for non-blocking operations
- **JSON API** - RESTful endpoints with Jackson serialization
- **Connection Pooling** - Built-in connection management
- **Batch Processing** - Efficient bulk user creation

## 📡 API Endpoints

### Health Check
```http
GET /health
```
Returns: `"UserTokenApi Java server is running"`

### User Authentication
```http
POST /api/auth/get-user-token
Content-Type: application/json

{
  "Username": "user1@example.com",
  "HashedPassword": "0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"
}
```

**Success Response:**
```json
{
  "success": true,
  "userId": 1,
  "errorMessage": null
}
```

### Database Setup
```http
GET /api/auth/create-db
```
Creates 10,000 test users in the database.

## 🛠️ Prerequisites

- **Java 17+** - Required for Spring Boot 3.2
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

## 🧪 Testing

### Quick Health Test
```bash
curl http://localhost:6000/health
```

### Setup Database
```bash
curl http://localhost:6000/api/auth/create-db
```

### Test Authentication
```bash
curl -X POST http://localhost:6000/api/auth/get-user-token \
  -H "Content-Type: application/json" \
  -d '{"Username":"user1@example.com","HashedPassword":"0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e"}'
```

## ⚡ Performance Characteristics

### Java/Spring Boot Features:
- **JVM Optimization** - HotSpot JIT compilation for runtime optimization
- **Thread Management** - Built-in thread pool management
- **Memory Management** - Garbage collection optimized for throughput
- **Connection Pooling** - Efficient database connection reuse
- **Async Processing** - CompletableFuture for non-blocking operations

### Expected Performance:
Based on similar JVM-based frameworks, expecting performance between .NET Core and Node.js:
- **Target**: ~3,000-5,000 req/s
- **Advantages**: JIT optimization, mature ecosystem
- **Considerations**: JVM startup time, garbage collection pauses

## 🏗️ Architecture

```
src/main/java/com/example/usertokenapi/
├── UserTokenApiApplication.java    # Main Spring Boot application
├── controller/
│   ├── AuthController.java         # Authentication endpoints
│   └── HealthController.java       # Health check endpoint
├── service/
│   └── DatabaseService.java        # Database operations
├── model/
│   └── User.java                   # User entity
└── dto/
    ├── LoginRequest.java           # Request DTO
    └── LoginResponse.java          # Response DTO
```

## 🔧 Configuration

Edit `src/main/resources/application.properties`:

```properties
server.port=6000
app.database.path=users.db
logging.level.com.example.usertokenapi=INFO
```

## 📊 Comparison with Other Languages

| Language | Framework | Expected RPS | Runtime |
|----------|-----------|--------------|---------|
| Rust | Actix-web | 17,887 | Native |
| .NET Core | ASP.NET | 7,417 | JIT |
| **Java** | **Spring Boot** | **~4,000** | **JVM** |
| Node.js | Express | 2,076 | V8 |
| Python | FastAPI | 1,850 | Interpreter |
| PHP | Vanilla | 1,227 | Interpreter |

Java's performance typically falls between .NET Core and Node.js, benefiting from JVM optimizations while carrying the overhead of garbage collection and framework abstractions.

## 🚀 Integration with Load Tester

The Java API integrates with the existing C# load tester. Update the load tester to include:

```csharp
private const string javaApiUrl = "http://localhost:6000/api/auth/get-user-token";
```

The API uses the same JSON contract as other implementations, ensuring compatibility with all testing tools.
