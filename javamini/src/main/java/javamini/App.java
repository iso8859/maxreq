package javamini;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.Headers;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpHandler;
import com.sun.net.httpserver.HttpServer;

import java.io.*;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.sql.*;
import java.time.Duration;
import java.util.*;

public class App {
    static class LoginRequest {
        public String username;
        public String hashedPassword;
    }
    static class LoginResponse {
        public boolean success;
        public Long userId;
        public String errorMessage;
    }

    static class DatabaseService implements AutoCloseable {
        private final boolean readOnly;
        private final String readUrl;
        private final String writeUrl;

        DatabaseService(String dbPath, boolean readOnly) throws Exception {
            this.readOnly = readOnly;
            String fileUri = dbPath.startsWith("file:") ? dbPath : ("file:" + dbPath);
            String roUri = fileUri + (fileUri.contains("?") ? "&" : "?") + "mode=ro";
            this.readUrl = "jdbc:sqlite:" + (readOnly ? roUri : fileUri);
            this.writeUrl = "jdbc:sqlite:" + fileUri;

            if (!readOnly) {
                try (Connection c = DriverManager.getConnection(writeUrl)) {
                    try (Statement s = c.createStatement()) {
                        s.execute("CREATE TABLE IF NOT EXISTS user (id INTEGER PRIMARY KEY AUTOINCREMENT, mail TEXT NOT NULL UNIQUE, hashed_password TEXT NOT NULL)");
                    }
                }
            }
        }

        Long getUser(String mail, String hashed) throws SQLException {
            try (Connection c = DriverManager.getConnection(readUrl);
                 PreparedStatement ps = c.prepareStatement("SELECT id FROM user WHERE mail = ? AND hashed_password = ?")) {
                ps.setString(1, mail);
                ps.setString(2, hashed);
                try (ResultSet rs = ps.executeQuery()) {
                    if (rs.next()) return rs.getLong(1);
                }
            }
            return null;
        }

        int seed(int count) throws SQLException {
            try (Connection c = DriverManager.getConnection(writeUrl)) {
                c.setAutoCommit(false);
                try (Statement s = c.createStatement()) {
                    s.execute("DELETE FROM user");
                }
                try (PreparedStatement ps = c.prepareStatement("INSERT INTO user (mail, hashed_password) VALUES (?, ?)")) {
                    for (int i = 1; i <= count; i++) {
                        String email = "user" + i + "@example.com";
                        String pwd = "password" + i;
                        String hashed = sha256(pwd);
                        ps.setString(1, email);
                        ps.setString(2, hashed);
                        ps.addBatch();
                        if (i % 1000 == 0) ps.executeBatch();
                    }
                    ps.executeBatch();
                }
                c.commit();
            }
            return count;
        }

        @Override public void close() { /* nothing persistent to close */ }
    }

    static String sha256(String s) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] out = md.digest(s.getBytes(StandardCharsets.UTF_8));
            StringBuilder sb = new StringBuilder();
            for (byte b : out) sb.append(String.format("%02x", b));
            return sb.toString();
        } catch (Exception e) { throw new RuntimeException(e); }
    }

    static void sendJson(HttpExchange ex, int code, Object obj) throws IOException {
        ObjectMapper om = new ObjectMapper();
        byte[] body = om.writeValueAsBytes(obj);
        Headers h = ex.getResponseHeaders();
        h.set("Content-Type", "application/json");
        ex.sendResponseHeaders(code, body.length);
        try (OutputStream os = ex.getResponseBody()) { os.write(body); }
    }

    static void sendText(HttpExchange ex, int code, String text) throws IOException {
        byte[] body = text.getBytes(StandardCharsets.UTF_8);
        ex.getResponseHeaders().set("Content-Type", "text/plain; charset=utf-8");
        ex.sendResponseHeaders(code, body.length);
        try (OutputStream os = ex.getResponseBody()) { os.write(body); }
    }

    public static void main(String[] args) throws Exception {
        String portStr = System.getenv().getOrDefault("PORT", "6000");
        int port = Integer.parseInt(portStr);
        String dbPath = System.getenv().getOrDefault("DB_PATH", "users.db");
        boolean readOnly = Boolean.parseBoolean(System.getenv().getOrDefault("READ_ONLY_DB", System.getenv().getOrDefault("READ_ONLY", "false")));
        int seedCount = Integer.parseInt(System.getenv().getOrDefault("SEED_USER_COUNT", "10000"));

        DatabaseService db = new DatabaseService(dbPath, readOnly);
        HttpServer server = HttpServer.create(new InetSocketAddress(port), 0);

        server.createContext("/api/auth/health", ex -> sendText(ex, 200, "UserTokenApi JavaMini server is running"));
        server.createContext("/api/auth/get-user-token", ex -> {
                if (!"POST".equalsIgnoreCase(ex.getRequestMethod())) { sendText(ex, 405, "Method Not Allowed"); return; }
                ObjectMapper om = new ObjectMapper().configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
                try (InputStream is = ex.getRequestBody()) {
                    LoginRequest req = om.readValue(is, LoginRequest.class);
                    if (req == null || req.username == null || req.hashedPassword == null) {
                        LoginResponse resp = new LoginResponse();
                        resp.success = false; resp.userId = null; resp.errorMessage = "username and hashed password are required";
                        sendJson(ex, 200, resp); return;
                    }
                    Long id = db.getUser(req.username, req.hashedPassword);
                    LoginResponse resp = new LoginResponse();
                    if (id != null) { resp.success = true; resp.userId = id; resp.errorMessage = null; }
                    else { resp.success = false; resp.userId = null; resp.errorMessage = "Invalid username or password"; }
                    sendJson(ex, 200, resp);
                } catch (Exception e) {
                    LoginResponse resp = new LoginResponse();
                    resp.success = false; resp.userId = null; resp.errorMessage = "An error occurred during authentication";
                    sendJson(ex, 200, resp);
                }
            });
        server.createContext("/api/auth/create-db", ex -> {
                if (!"GET".equalsIgnoreCase(ex.getRequestMethod())) { sendText(ex, 405, "Method Not Allowed"); return; }
                if (readOnly) { sendJson(ex, 403, Map.of("error", "Database opened in read-only mode")); return; }
                long start = System.nanoTime();
                try {
                    int count = db.seed(seedCount);
                    long durMs = Duration.ofNanos(System.nanoTime() - start).toMillis();
                    sendText(ex, 200, "Successfully created " + count + " users in " + durMs + "ms");
                } catch (Exception e) {
                    sendJson(ex, 500, Map.of("error", "An error occurred while creating the database"));
                }
            });

        server.setExecutor(java.util.concurrent.Executors.newFixedThreadPool(Runtime.getRuntime().availableProcessors()));
        System.out.println("Starting JavaMini server on http://localhost:" + port + " using DB '" + dbPath + "' " + (readOnly ? "(read-only; seeding disabled)" : "(writable)"));
        System.out.println("  GET  /api/auth/health");
        System.out.println("  POST /api/auth/get-user-token");
        System.out.println("  GET  /api/auth/create-db");

        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            System.out.println("Shutting down JavaMini...");
            server.stop(0);
            try { db.close(); } catch (Exception ignored) {}
        }));

        server.start();
        try { while (true) { Thread.sleep(1_000_000L); } } catch (InterruptedException ignored) {}
    }
}
