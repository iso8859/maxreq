#include <drogon/drogon.h>
#include <sqlite3.h>
#include <string>
#include <iostream>
#include <memory>
#include <sstream>
#include <iomanip>
#include <openssl/sha.h>
#include <json/json.h>

// SQLite wrapper for RAII
class Database {
private:
    sqlite3* db;
    
public:
    Database(const std::string& path) : db(nullptr) {
        int rc = sqlite3_open(path.c_str(), &db);
        if (rc) {
            std::cerr << "Can't open database: " << sqlite3_errmsg(db) << std::endl;
            sqlite3_close(db);
            db = nullptr;
        }
    }
    
    ~Database() {
        if (db) {
            sqlite3_close(db);
        }
    }
    
    bool isValid() const { return db != nullptr; }
    
    bool execute(const std::string& sql) {
        char* errmsg = nullptr;
        int rc = sqlite3_exec(db, sql.c_str(), nullptr, nullptr, &errmsg);
        if (rc != SQLITE_OK) {
            std::cerr << "SQL error: " << errmsg << std::endl;
            sqlite3_free(errmsg);
            return false;
        }
        return true;
    }
    
    struct User {
        long id;
        std::string mail;
        std::string hashedPassword;
    };
    
    std::unique_ptr<User> getUserByCredentials(const std::string& mail, const std::string& hashedPassword) {
        sqlite3_stmt* stmt;
        const char* sql = "SELECT id FROM user WHERE mail = ? AND hashed_password = ?";
        
        int rc = sqlite3_prepare_v2(db, sql, -1, &stmt, nullptr);
        if (rc != SQLITE_OK) {
            std::cerr << "Failed to prepare statement: " << sqlite3_errmsg(db) << std::endl;
            return nullptr;
        }
        
        sqlite3_bind_text(stmt, 1, mail.c_str(), -1, SQLITE_STATIC);
        sqlite3_bind_text(stmt, 2, hashedPassword.c_str(), -1, SQLITE_STATIC);
        
        std::unique_ptr<User> user = nullptr;
        if (sqlite3_step(stmt) == SQLITE_ROW) {
            user = std::make_unique<User>();
            user->id = sqlite3_column_int64(stmt, 0);
        }
        
        sqlite3_finalize(stmt);
        return user;
    }
    
    int createTestUsers(int count) {
        // Clear existing users
        if (!execute("DELETE FROM user")) {
            return 0;
        }
        
        // Begin transaction
        if (!execute("BEGIN TRANSACTION")) {
            return 0;
        }
        
        sqlite3_stmt* stmt;
        const char* sql = "INSERT INTO user (mail, hashed_password) VALUES (?, ?)";

        int rc = sqlite3_prepare_v2(db, sql, -1, &stmt, nullptr);
        if (rc != SQLITE_OK) {
            execute("ROLLBACK");
            return 0;
        }
        
        int inserted = 0;
        for (int i = 1; i <= count; i++) {
            std::string email = "user" + std::to_string(i) + "@example.com";
            std::string password = "password" + std::to_string(i);
            std::string hashedPassword = computeSha256Hash(password);
            
            sqlite3_bind_text(stmt, 1, email.c_str(), -1, SQLITE_TRANSIENT);
            sqlite3_bind_text(stmt, 2, hashedPassword.c_str(), -1, SQLITE_TRANSIENT);
            
            if (sqlite3_step(stmt) == SQLITE_DONE) {
                inserted++;
            }
            
            sqlite3_reset(stmt);
        }
        
        sqlite3_finalize(stmt);
        
        if (execute("COMMIT")) {
            return inserted;
        } else {
            execute("ROLLBACK");
            return 0;
        }
    }
    
private:
    std::string computeSha256Hash(const std::string& input) {
        unsigned char hash[SHA256_DIGEST_LENGTH];
        SHA256((unsigned char*)input.c_str(), input.length(), hash);
        
        std::stringstream ss;
        for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
            ss << std::hex << std::setw(2) << std::setfill('0') << (int)hash[i];
        }
        return ss.str();
    }
};

// Initialize database with required table
bool initializeDatabase(const std::string& dbPath) {
    Database db(dbPath);
    if (!db.isValid()) {
        return false;
    }
    
    const char* createTableSql = R"(
        CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mail TEXT NOT NULL UNIQUE,
            hashed_password TEXT NOT NULL
        )
    )";
    
    return db.execute(createTableSql);
}

int main() {
    const std::string dbPath = "users.db";
    const int port = 8081;

    // Initialize database
    if (!initializeDatabase(dbPath)) {
        std::cerr << "Failed to initialize database" << std::endl;
        return 1;
    }

    drogon::app()
        .setLogPath("./")
        .setLogLevel(trantor::Logger::kWarn)
        .setThreadNum(0);

    // Health check endpoint
    drogon::app().registerHandler("/health",
        [](const drogon::HttpRequestPtr& req,
           std::function<void(const drogon::HttpResponsePtr &)> &&callback) {
            auto resp = drogon::HttpResponse::newHttpResponse();
            resp->setContentTypeString("text/plain");
            resp->setBody("UserTokenApi C++ server is running");
            callback(resp);
        },
        {drogon::Get});

    // Authentication endpoint
    drogon::app().registerHandler("/api/auth/get-user-token",
        [dbPath](const drogon::HttpRequestPtr& req,
                 std::function<void(const drogon::HttpResponsePtr &)> &&callback) {
            try {
                Json::Value requestBody;
                Json::CharReaderBuilder builder;
                std::string errs;
                std::string body(req->body());
				std::stringstream s(body);
                if (!Json::parseFromStream(builder, s, &requestBody, &errs)) {
                    Json::Value response;
                    response["success"] = false;
                    response["userId"] = Json::nullValue;
                    response["errorMessage"] = "Invalid JSON";
                    auto resp = drogon::HttpResponse::newHttpJsonResponse(response);
                    callback(resp);
                    return;
                }
                std::string username = requestBody["Username"].asString();
                std::string hashedPassword = requestBody["HashedPassword"].asString();

                Database db(dbPath);
                Json::Value response;
                if (!db.isValid()) {
                    response["success"] = false;
                    response["userId"] = Json::nullValue;
                    response["errorMessage"] = "Database connection failed";
                } else {
                    auto user = db.getUserByCredentials(username, hashedPassword);
                    if (user) {
                        response["success"] = true;
                        response["userId"] = static_cast<Json::Int64>(user->id);
                        response["errorMessage"] = Json::nullValue;
                    } else {
                        response["success"] = false;
                        response["userId"] = Json::nullValue;
                        response["errorMessage"] = "Invalid username or password";
                    }
                }
                auto resp = drogon::HttpResponse::newHttpJsonResponse(response);
                callback(resp);
            } catch (const std::exception& e) {
                Json::Value response;
                Json::CharReaderBuilder builder;
                response["success"] = false;
                response["userId"] = Json::nullValue;
                response["errorMessage"] = "An error occurred during authentication";
                auto resp = drogon::HttpResponse::newHttpJsonResponse(response);
                callback(resp);
            }
        },
        {drogon::Post});

    // Database creation endpoint
    drogon::app().registerHandler("/api/auth/create-db",
        [dbPath](const drogon::HttpRequestPtr& req,
                 std::function<void(const drogon::HttpResponsePtr &)> &&callback) {
            auto resp = drogon::HttpResponse::newHttpResponse();
            try {
                Database db(dbPath);
                if (!db.isValid()) {
                    resp->setStatusCode(drogon::k500InternalServerError);
                    resp->setContentTypeString("text/plain");
                    resp->setBody("Database connection failed");
                    callback(resp);
                    return;
                }
                int count = db.createTestUsers(10000);
                if (count > 0) {
                    std::string message = "Successfully created " + std::to_string(count) + " users in the database";
                    resp->setBody(message);
                } else {
                    resp->setStatusCode(drogon::k500InternalServerError);
                    resp->setBody("Failed to create users");
                }
                resp->setContentTypeString("text/plain");
                callback(resp);
            } catch (const std::exception& e) {
                resp->setStatusCode(drogon::k500InternalServerError);
                resp->setContentTypeString("text/plain");
                resp->setBody("An error occurred while creating the database");
                callback(resp);
            }
        },
        {drogon::Get});

    std::cout << "Starting C++ UserTokenApi server on http://localhost:" << port << std::endl;
    std::cout << "Available endpoints:" << std::endl;
    std::cout << "  GET /health - Health check" << std::endl;
    std::cout << "  POST /api/auth/get-user-token - Authenticate user" << std::endl;
    std::cout << "  GET /api/auth/create-db - Create test database with 10,000 users" << std::endl;

    drogon::app().addListener("0.0.0.0", port);
    drogon::app().run();
    return 0;
}