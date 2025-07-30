#include <httplib.h>
#include <nlohmann/json.hpp>

int main() {
    httplib::Server server;

    server.Get("/health", [](const httplib::Request&, httplib::Response& res) {
        res.set_content("UserTokenApi C++ server is running", "text/plain");
        });

    server.Post("/api/auth/get-user-token", [](const httplib::Request& req, httplib::Response& res) {
        // Your authentication logic here
        nlohmann::json response = { {"success", true}, {"userId", 1} };
        res.set_content(response.dump(), "application/json");
        });

    server.listen("0.0.0.0", 808);
}