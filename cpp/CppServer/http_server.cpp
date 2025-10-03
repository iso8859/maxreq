/*!
	\file http_server.cpp
	\brief HTTP server example
	\author Ivan Shynkarenka
	\date 30.04.2019
	\copyright MIT License
*/

#include "sdkddkver.h"
#define FMT_UNICODE 0

#include "asio_service.h"
#include <windows.h>

#include "server/http/http_server.h"
#include "string/string_utils.h"
#include "utility/singleton.h"

#include <iostream>
#include <map>
#include <mutex>

#include "asio/impl/src.hpp"
// #include "asio/impl/error.ipp"

#include "sqlite3.h"
#include "rapidjson/document.h"
#include "rapidjson/writer.h"
#include "rapidjson/stringbuffer.h"
// Global sqlite database pointer
sqlite3* g_db = nullptr;

class Cache : public CppCommon::Singleton<Cache>
{
	friend CppCommon::Singleton<Cache>;

public:
	std::string GetAllCache()
	{
		std::scoped_lock locker(_cache_lock);
		std::string result;
		result += "[\n";
		for (const auto& item : _cache)
		{
			result += "  {\n";
			result += "    \"key\": \"" + item.first + "\",\n";
			result += "    \"value\": \"" + item.second + "\",\n";
			result += "  },\n";
		}
		result += "]\n";
		return result;
	}

	bool GetCacheValue(std::string_view key, std::string& value)
	{
		std::scoped_lock locker(_cache_lock);
		auto it = _cache.find(key);
		if (it != _cache.end())
		{
			value = it->second;
			return true;
		}
		else
			return false;
	}

	void PutCacheValue(std::string_view key, std::string_view value)
	{
		std::scoped_lock locker(_cache_lock);
		auto it = _cache.emplace(key, value);
		if (!it.second)
			it.first->second = value;
	}

	bool DeleteCacheValue(std::string_view key, std::string& value)
	{
		std::scoped_lock locker(_cache_lock);
		auto it = _cache.find(key);
		if (it != _cache.end())
		{
			value = it->second;
			_cache.erase(it);
			return true;
		}
		else
			return false;
	}

private:
	std::mutex _cache_lock;
	std::map<std::string, std::string, std::less<>> _cache;
};

struct User {
	long id;
	std::string mail;
	std::string hashedPassword;
};

struct sqlitePreparedStatement
{
	bool available;
	sqlite3* db;
	sqlite3_stmt* stmt;
};

std::vector<sqlitePreparedStatement> _preparedStatements;
const char* sql = "SELECT id FROM user WHERE mail = ? AND hashed_password = ?";

std::mutex preparedStatementsMutex;

class HTTPCacheSession : public CppServer::HTTP::HTTPSession
{

public:
	using CppServer::HTTP::HTTPSession::HTTPSession;

protected:
	std::unique_ptr<User> getUserByCredentials(const std::string& mail, const std::string& hashedPassword) {
		// Find prepared statement in the cache
		// Make this search thread safe
		// Use Win32 critical section or mutex
		std::vector<sqlitePreparedStatement>::iterator it = _preparedStatements.end();
		{
			std::scoped_lock locker(preparedStatementsMutex);
			// find first available prepared statement available == true
			it = std::find_if(_preparedStatements.begin(), _preparedStatements.end(),
				[](const sqlitePreparedStatement& ps) { return ps.available; });
		}
		if (it == _preparedStatements.end())
		{
			// Create new prepared statement
			sqlitePreparedStatement ps;
			ps.available = false;
			// Open read only connection to the database
			sqlite3_open_v2("users.db", &ps.db, SQLITE_OPEN_READONLY, nullptr);
			sqlite3_prepare_v2(ps.db, sql, -1, &ps.stmt, nullptr);
			_preparedStatements.push_back(ps);
			it = _preparedStatements.end() - 1;
			std::cout << "Prepared statements in cache: " << _preparedStatements.size() << std::endl;
		}

		sqlite3_stmt* stmt = it->stmt;

		sqlite3_bind_text(stmt, 1, mail.c_str(), -1, SQLITE_STATIC);
		sqlite3_bind_text(stmt, 2, hashedPassword.c_str(), -1, SQLITE_STATIC);

		std::unique_ptr<User> user = nullptr;
		if (sqlite3_step(stmt) == SQLITE_ROW) {
			user = std::make_unique<User>();
			user->id = (long)sqlite3_column_int64(stmt, 0);
		}

		sqlite3_reset(stmt);

		it->available = true;

		return user;
	}

	void onReceivedRequest(const CppServer::HTTP::HTTPRequest& request) override
	{
		// Show HTTP request content
		// std::cout << std::endl << request;
		auto method = request.url();
		// std::cout << "Method: " << method << std::endl;
		// if (request.method() == "POST")
		{
			if (method == "/api/auth/get-user-token")
			{
				// Quick parse JSON body (not secure, just for demo purposes)
				std::string errs;
				std::string body(request.body());
				rapidjson::Document d;
				if (d.Parse(body.c_str()).HasParseError())
				{					
					SendResponseAsync(response().MakeErrorResponse("Invalid JSON body", "text/plain"));
					return;
				}
				if (!d.HasMember("UserName") || !d["UserName"].IsString() ||
					!d.HasMember("HashedPassword") || !d["HashedPassword"].IsString())
				{
					SendResponseAsync(response().MakeErrorResponse("Invalid JSON body", "text/plain"));
					return;
				}
				std::string username = d["UserName"].GetString();
				std::string hashedPassword = d["HashedPassword"].GetString();

				if (username == "no_db")
				{
					// no db request
					rapidjson::StringBuffer s;
					rapidjson::Writer<rapidjson::StringBuffer> writer(s);
					writer.StartObject();
					writer.Key("Success");
					writer.Bool(true);
					writer.Key("UserId");
					writer.Int64(1);
					writer.EndObject();
					SendResponseAsync(response().MakeGetResponse(s.GetString(), "application/json"));
					return;
				}
				// Find user in the database
				auto user = getUserByCredentials(username, hashedPassword);
				if (user)
				{
					// Return the token as JSON
					rapidjson::StringBuffer s;
					rapidjson::Writer<rapidjson::StringBuffer> writer(s);
					writer.StartObject();
					writer.Key("Success");
					writer.Bool(true);
					writer.Key("UserId");
					writer.Int64(user->id);
					writer.EndObject();
					SendResponseAsync(response().MakeGetResponse(s.GetString(), "application/json"));
				}
				else
				{
					SendResponseAsync(response().MakeErrorResponse("Invalid credentials", "text/plain"));
				}
			}
			else if (method == "/api/auth/create-db")
			{
				// Fill user database with test users
				int count = 0;
				if (g_db)
				{
					HCRYPTPROV hCryptProv;
					HCRYPTHASH hHash;
					CryptAcquireContext(&hCryptProv, nullptr, nullptr, PROV_RSA_AES, CRYPT_VERIFYCONTEXT);
					// Purge existing users
					sqlite3_exec(g_db, "DELETE FROM user;", nullptr, nullptr, nullptr);
					sqlite3_exec(g_db, "BEGIN TRANSACTION;", nullptr, nullptr, nullptr);
					char buffer[256];
					sqlite3_stmt* stmt;
					const char* sql = "INSERT INTO user (mail, hashed_password) VALUES (?, ?)";
					if (sqlite3_prepare_v2(g_db, sql, -1, &stmt, nullptr) == SQLITE_OK)
					{
						for (int i = 0; i < 10000; ++i)
						{
							std::string email = "user" + std::to_string(i + 1) + "@example.com";
							std::string password = "password" + std::to_string(i + 1);
							// Use Win32 SHA256 
							if (!CryptCreateHash(hCryptProv, CALG_SHA_256, 0, 0, &hHash))
							{
								std::cerr << "CryptCreateHash failed: " << GetLastError() << std::endl;
							}

							if (CryptHashData(
								hHash,
								reinterpret_cast<const BYTE*>(password.data()),
								static_cast<DWORD>(password.size()),
								0))
							{
								BYTE hash[32];
								DWORD hashSize = sizeof(hash);
								if (CryptGetHashParam(
									hHash,
									HP_HASHVAL,
									hash,
									&hashSize,
									0))
								{
									for (DWORD j = 0; j < hashSize; ++j)
									{
										sprintf_s(buffer + j * 2, 256 - j * 2, "%02x", hash[j]);
									}
									sqlite3_bind_text(stmt, 1, email.c_str(), -1, SQLITE_STATIC);
									sqlite3_bind_text(stmt, 2, buffer, -1, SQLITE_STATIC);
									if (sqlite3_step(stmt) == SQLITE_DONE)
									{
										++count;
									}
									else
									{
										std::cerr << "Failed to insert user: " << sqlite3_errmsg(g_db) << std::endl;
									}
									sqlite3_reset(stmt);
								}
							}
							CryptDestroyHash(hHash);
						}
					}
					sqlite3_finalize(stmt);
					sqlite3_exec(g_db, "COMMIT;", nullptr, nullptr, nullptr);
					CryptReleaseContext(hCryptProv, 0);
				}

				std::string message = "Successfully created " + std::to_string(count) + " users in the database";
				SendResponseAsync(response().MakeGetResponse(message, "text/plain"));
			}
		}
	}

	void onReceivedRequestError(const CppServer::HTTP::HTTPRequest& request, const std::string& error) override
	{
		std::cout << "Request error: " << error << std::endl;
	}

	void onError(int error, const std::string& category, const std::string& message) override
	{
		std::cout << "HTTP session caught an error with code " << error << " and category '" << category << "': " << message << std::endl;
	}
};

class HTTPCacheServer : public CppServer::HTTP::HTTPServer
{
public:
	using CppServer::HTTP::HTTPServer::HTTPServer;

protected:
	std::shared_ptr<CppServer::Asio::TCPSession> CreateSession(const std::shared_ptr<CppServer::Asio::TCPServer>& server) override
	{
		return std::make_shared<HTTPCacheSession>(std::dynamic_pointer_cast<CppServer::HTTP::HTTPServer>(server));
	}

protected:
	void onError(int error, const std::string& category, const std::string& message) override
	{
		std::cout << "HTTP server caught an error with code " << error << " and category '" << category << "': " << message << std::endl;
	}
};

int main(int argc, char** argv)
{
	auto rc = sqlite3_open("users.db", &g_db);
	if (rc)
	{
		std::cerr << "Can't open database: " << sqlite3_errmsg(g_db) << std::endl;
		sqlite3_close(g_db);
		return 1;
	}

	// Create the users table if it doesn't exist
	rc = sqlite3_exec(g_db,
		"CREATE TABLE IF NOT EXISTS user ("
		"id INTEGER PRIMARY KEY AUTOINCREMENT,"
		"mail TEXT NOT NULL UNIQUE,"
		"hashed_password TEXT NOT NULL"
		");",
		nullptr, nullptr, nullptr);
	if (rc != SQLITE_OK)
	{
		std::cerr << "SQL error: " << sqlite3_errmsg(g_db) << std::endl;
		sqlite3_close(g_db);
		return 1;
	}
	std::cout << "Database initialized successfully." << std::endl;
	// HTTP server port
	int port = 8081;
	if (argc > 1)
		port = std::atoi(argv[1]);
	// HTTP server content path
	std::string www = "../www/api";
	if (argc > 2)
		www = argv[2];

	std::cout << "HTTP server port: " << port << std::endl;
	std::cout << "HTTP server static content path: " << www << std::endl;
	std::cout << "HTTP server website: " << "http://localhost:" << port << "/api/index.html" << std::endl;

	std::cout << std::endl;

	// Create a new Asio service
	auto service = std::make_shared<AsioService>();

	// Start the Asio service
	std::cout << "Asio service starting...";
	service->Start();
	std::cout << "Done!" << std::endl;

	// Create a new HTTP server
	auto server = std::make_shared<HTTPCacheServer>(service, port);
	// server->AddStaticContent(www, "/api");
	// Start the server
	std::cout << "Server starting...";
	server->Start();
	std::cout << "Done!" << std::endl;

	std::cout << "Press Enter to stop the server or '!' to restart the server..." << std::endl;

	// Perform text input
	std::string line;
	while (getline(std::cin, line))
	{
		if (line.empty())
			break;

		// Restart the server
		if (line == "!")
		{
			std::cout << "Server restarting...";
			server->Restart();
			std::cout << "Done!" << std::endl;
			continue;
		}
	}

	// Stop the server
	std::cout << "Server stopping...";
	server->Stop();
	std::cout << "Done!" << std::endl;

	// Stop the Asio service
	std::cout << "Asio service stopping...";
	service->Stop();
	std::cout << "Done!" << std::endl;

	return 0;
}
