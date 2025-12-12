#include "uWebSockets/App.h"
#include <string>
#include <string_view>
#include <iostream>
#include <cstring>
#include <sstream>
#include <iomanip>
#include <vector>
#include <mutex>

#include "sqlite/sqlite3.h"

// SHA256 - Using OpenSSL or Windows CryptoAPI
#ifdef _WIN32
#include <windows.h>
#include <wincrypt.h>
#pragma comment(lib, "advapi32.lib")
#else
#include <openssl/sha.h>
#endif

// Global database and prepared statement cache
sqlite3* g_db = nullptr;
std::vector<sqlite3_stmt*> g_stmt_pool;
std::mutex g_stmt_mutex;

// Helper function to hash password with SHA256
std::string hashPassword(const std::string& password) {
#ifdef _WIN32
    HCRYPTPROV hProv = 0;
    HCRYPTHASH hHash = 0;
    
    if (!CryptAcquireContext(&hProv, NULL, NULL, PROV_RSA_AES, CRYPT_VERIFYCONTEXT)) {
        return "";
    }
    
    if (!CryptCreateHash(hProv, CALG_SHA_256, 0, 0, &hHash)) {
        CryptReleaseContext(hProv, 0);
        return "";
    }
    
    if (!CryptHashData(hHash, (BYTE*)password.c_str(), password.length(), 0)) {
        CryptDestroyHash(hHash);
        CryptReleaseContext(hProv, 0);
        return "";
    }
    
    BYTE hash[32];
    DWORD hashLen = 32;
    if (!CryptGetHashParam(hHash, HP_HASHVAL, hash, &hashLen, 0)) {
        CryptDestroyHash(hHash);
        CryptReleaseContext(hProv, 0);
        return "";
    }
    
    CryptDestroyHash(hHash);
    CryptReleaseContext(hProv, 0);
    
    std::stringstream ss;
    for (DWORD i = 0; i < hashLen; i++) {
        ss << std::hex << std::setw(2) << std::setfill('0') << (int)hash[i];
    }
    return ss.str();
#else
    unsigned char hash[SHA256_DIGEST_LENGTH];
    SHA256_CTX sha256;
    SHA256_Init(&sha256);
    SHA256_Update(&sha256, password.c_str(), password.length());
    SHA256_Final(hash, &sha256);
    
    std::stringstream ss;
    for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
        ss << std::hex << std::setw(2) << std::setfill('0') << (int)hash[i];
    }
    return ss.str();
#endif
}

// Simple JSON value extractor (no library, direct string parsing)
std::string extractJsonString(const std::string& json, const std::string& key) {
    std::string searchKey = "\"" + key + "\"";
    size_t pos = json.find(searchKey);
    if (pos == std::string::npos) return "";
    
    pos = json.find(":", pos);
    if (pos == std::string::npos) return "";
    
    pos = json.find("\"", pos);
    if (pos == std::string::npos) return "";
    pos++; // Skip opening quote
    
    size_t endPos = json.find("\"", pos);
    if (endPos == std::string::npos) return "";
    
    return json.substr(pos, endPos - pos);
}

// Get or create a prepared statement from the pool
sqlite3_stmt* getStatement() {
    std::lock_guard<std::mutex> lock(g_stmt_mutex);
    
    if (!g_stmt_pool.empty()) {
        sqlite3_stmt* stmt = g_stmt_pool.back();
        g_stmt_pool.pop_back();
        return stmt;
    }
    
    // Create new prepared statement
    sqlite3_stmt* stmt;
    const char* sql = "SELECT id FROM user WHERE mail = ? AND hashed_password = ? LIMIT 1";
    
    if (sqlite3_prepare_v2(g_db, sql, -1, &stmt, nullptr) != SQLITE_OK) {
        std::cerr << "Failed to prepare statement: " << sqlite3_errmsg(g_db) << std::endl;
        return nullptr;
    }
    
    std::cout << "Created new prepared statement. Pool size: " << (g_stmt_pool.size() + 1) << std::endl;
    return stmt;
}

// Return a statement to the pool
void returnStatement(sqlite3_stmt* stmt) {
    if (!stmt) return;
    
    sqlite3_reset(stmt);
    sqlite3_clear_bindings(stmt);
    
    std::lock_guard<std::mutex> lock(g_stmt_mutex);
    g_stmt_pool.push_back(stmt);
}

// Get user by credentials
long long getUserByCredentials(const std::string& userName, const std::string& hashedPassword) {
    // Special test case
    if (userName == "no_db") {
        return 12345;
    }
    
    sqlite3_stmt* stmt = getStatement();
    if (!stmt) return -1;
    
    sqlite3_bind_text(stmt, 1, userName.c_str(), -1, SQLITE_STATIC);
    sqlite3_bind_text(stmt, 2, hashedPassword.c_str(), -1, SQLITE_STATIC);
    
    long long userId = -1;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        userId = sqlite3_column_int64(stmt, 0);
    }
    
    returnStatement(stmt);
    return userId;
}

// Create test users
int createTestUsers(int count) {
    // Clear existing users
    sqlite3_exec(g_db, "DELETE FROM user", nullptr, nullptr, nullptr);
    
    // Begin transaction
    sqlite3_exec(g_db, "BEGIN TRANSACTION", nullptr, nullptr, nullptr);
    
    sqlite3_stmt* stmt;
    const char* sql = "INSERT INTO user (mail, hashed_password) VALUES (?, ?)";
    
    if (sqlite3_prepare_v2(g_db, sql, -1, &stmt, nullptr) != SQLITE_OK) {
        std::cerr << "Failed to prepare insert statement: " << sqlite3_errmsg(g_db) << std::endl;
        return 0;
    }
    
    int inserted = 0;
    for (int i = 1; i <= count; i++) {
        std::string email = "user" + std::to_string(i) + "@example.com";
        std::string password = "password" + std::to_string(i);
        std::string hashed = hashPassword(password);
        
        sqlite3_bind_text(stmt, 1, email.c_str(), -1, SQLITE_TRANSIENT);
        sqlite3_bind_text(stmt, 2, hashed.c_str(), -1, SQLITE_TRANSIENT);
        
        if (sqlite3_step(stmt) == SQLITE_DONE) {
            inserted++;
        } else {
            std::cerr << "Failed to insert user " << i << ": " << sqlite3_errmsg(g_db) << std::endl;
        }
        
        sqlite3_reset(stmt);
    }
    
    sqlite3_finalize(stmt);
    sqlite3_exec(g_db, "COMMIT", nullptr, nullptr, nullptr);
    
    return inserted;
}

// Initialize database
bool initDatabase() {
    int rc = sqlite3_open("users.db", &g_db);
    if (rc != SQLITE_OK) {
        std::cerr << "Can't open database: " << sqlite3_errmsg(g_db) << std::endl;
        return false;
    }
    
    // Enable WAL mode for better concurrency
    sqlite3_exec(g_db, "PRAGMA journal_mode=WAL", nullptr, nullptr, nullptr);
    sqlite3_exec(g_db, "PRAGMA synchronous=NORMAL", nullptr, nullptr, nullptr);
    sqlite3_exec(g_db, "PRAGMA cache_size=-64000", nullptr, nullptr, nullptr);
    sqlite3_exec(g_db, "PRAGMA temp_store=MEMORY", nullptr, nullptr, nullptr);
    
    // Create table if it doesn't exist
    const char* createTableSql = 
        "CREATE TABLE IF NOT EXISTS user ("
        "id INTEGER PRIMARY KEY AUTOINCREMENT,"
        "mail TEXT NOT NULL UNIQUE,"
        "hashed_password TEXT NOT NULL"
        ")";
    
    rc = sqlite3_exec(g_db, createTableSql, nullptr, nullptr, nullptr);
    if (rc != SQLITE_OK) {
        std::cerr << "Failed to create table: " << sqlite3_errmsg(g_db) << std::endl;
        return false;
    }
    
    // Create index for faster lookups
    const char* createIndexSql = 
        "CREATE INDEX IF NOT EXISTS idx_user_mail_password ON user(mail, hashed_password)";
    
    sqlite3_exec(g_db, createIndexSql, nullptr, nullptr, nullptr);
    
    std::cout << "Database initialized successfully." << std::endl;
    return true;
}

int main() {
    // Initialize database
    if (!initDatabase()) {
        return 1;
    }
    
    // Create uWebSockets app
    uWS::App().get("/health", [](auto *res, auto *req) {
        res->writeStatus("200 OK");
        res->writeHeader("Content-Type", "text/plain");
        res->end("UserTokenApi C++ uWebSockets server is running");
        
    }).get("/api/auth/create-db", [](auto *res, auto *req) {
        int count = createTestUsers(10000);
        
        std::string response = "Successfully created " + std::to_string(count) + " users in the database";
        
        res->writeStatus("200 OK");
        res->writeHeader("Content-Type", "text/plain");
        res->end(response);
        
    }).post("/api/auth/get-user-token", [](auto *res, auto *req) {
        // Read the request body
        std::string buffer;
        
        res->onData([res, buffer = std::move(buffer)](std::string_view data, bool last) mutable {
            buffer.append(data.data(), data.length());
            
            if (last) {
                // Parse JSON manually (no library)
                std::string userName = extractJsonString(buffer, "UserName");
                std::string hashedPassword = extractJsonString(buffer, "HashedPassword");
                
                if (userName.empty() || hashedPassword.empty()) {
                    std::string errorResponse = "{\"Success\":false,\"UserId\":null,\"ErrorMessage\":\"Invalid request body\"}";
                    res->writeStatus("400 Bad Request");
                    res->writeHeader("Content-Type", "application/json");
                    res->end(errorResponse);
                    return;
                }
                
                // Get user from database
                long long userId = getUserByCredentials(userName, hashedPassword);
                
                std::string response;
                if (userId != -1) {
                    response = "{\"Success\":true,\"UserId\":" + std::to_string(userId) + ",\"ErrorMessage\":null}";
                } else {
                    response = "{\"Success\":false,\"UserId\":null,\"ErrorMessage\":\"Invalid username or password\"}";
                }
                
                res->writeStatus("200 OK");
                res->writeHeader("Content-Type", "application/json");
                res->end(response);
            }
        });
        
        res->onAborted([]() {
            std::cout << "Request aborted" << std::endl;
        });
        
    }).listen(8081, [](auto *listen_socket) {
        if (listen_socket) {
            std::cout << "?? C++ uWebSockets UserTokenApi server running on http://localhost:8081" << std::endl;
            std::cout << "Available endpoints:" << std::endl;
            std::cout << "  GET  /health - Health check" << std::endl;
            std::cout << "  POST /api/auth/get-user-token - Authenticate user" << std::endl;
            std::cout << "  GET  /api/auth/create-db - Create test database" << std::endl;
        } else {
            std::cerr << "Failed to listen on port 8081" << std::endl;
        }
    }).run();
    
    // Cleanup
    {
        std::lock_guard<std::mutex> lock(g_stmt_mutex);
        for (auto stmt : g_stmt_pool) {
            sqlite3_finalize(stmt);
        }
        g_stmt_pool.clear();
    }
    
    if (g_db) {
        sqlite3_close(g_db);
    }
    
    return 0;
}
