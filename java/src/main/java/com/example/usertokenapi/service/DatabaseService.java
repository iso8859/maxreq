package com.example.usertokenapi.service;

import com.example.usertokenapi.model.User;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import java.sql.*;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.atomic.AtomicInteger;

@Service
public class DatabaseService {

    @Value("${app.database.path:users.db}")
    private String databasePath;

    // Connection and PreparedStatement pool for performance optimization
    private final ConcurrentLinkedQueue<ConnectionPool> connectionPools = new ConcurrentLinkedQueue<>();
    private final AtomicInteger poolSize = new AtomicInteger(0);
    private final int MAX_POOL_SIZE = 10;

    // Inner class to hold connection and prepared statement together
    private static class ConnectionPool {
        final Connection connection;
        final PreparedStatement getUserStatement;

        ConnectionPool(Connection connection, PreparedStatement getUserStatement) {
            this.connection = connection;
            this.getUserStatement = getUserStatement;
        }

        void close() {
            try {
                if (getUserStatement != null) getUserStatement.close();
                if (connection != null) connection.close();
            } catch (SQLException e) {
                // Log error but don't throw
                System.err.println("Error closing connection pool: " + e.getMessage());
            }
        }
    }

    private String getConnectionString() {
        return "jdbc:sqlite:" + databasePath;
    }

    private ConnectionPool getOrCreateConnectionPool() throws SQLException {
        ConnectionPool pool = connectionPools.poll();
        
        if (pool == null) {
            // Create new connection and prepared statement
            Connection connection = DriverManager.getConnection(getConnectionString());
            PreparedStatement getUserStatement = connection.prepareStatement(
                "SELECT id FROM user WHERE mail = ? AND hashed_password = ? LIMIT 1"
            );
            
            pool = new ConnectionPool(connection, getUserStatement);
            poolSize.incrementAndGet();
            System.out.println("Created new connection pool. Total pools: " + poolSize.get());
        }
        
        return pool;
    }

    private void returnConnectionPool(ConnectionPool pool) {
        if (pool != null && connectionPools.size() < MAX_POOL_SIZE) {
            connectionPools.offer(pool);
        } else if (pool != null) {
            // Pool is full, close this connection
            pool.close();
            poolSize.decrementAndGet();
        }
    }

    @PostConstruct
    public void initializeDatabase() {
        try (Connection connection = DriverManager.getConnection(getConnectionString())) {
            String createTableSql = """
                CREATE TABLE IF NOT EXISTS user (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    mail TEXT NOT NULL UNIQUE,
                    hashed_password TEXT NOT NULL
                )
                """;
            
            try (Statement statement = connection.createStatement()) {
                statement.execute(createTableSql);
            }
        } catch (SQLException e) {
            throw new RuntimeException("Failed to initialize database", e);
        }
    }

    public CompletableFuture<User> getUserByMailAndPasswordAsync(String mail, String hashedPassword) {
        return CompletableFuture.supplyAsync(() -> {
            // Special case for testing (similar to C# and TypeScript versions)
            if ("no_db".equals(mail)) {
                User user = new User();
                user.setId(1L);
                return user;
            }

            ConnectionPool pool = null;
            try {
                pool = getOrCreateConnectionPool();
                
                // Reuse the prepared statement
                PreparedStatement statement = pool.getUserStatement;
                statement.setString(1, mail);
                statement.setString(2, hashedPassword);
                
                try (ResultSet resultSet = statement.executeQuery()) {
                    if (resultSet.next()) {
                        User user = new User();
                        user.setId(resultSet.getLong("id"));
                        return user;
                    }
                }
            } catch (SQLException e) {
                throw new RuntimeException("Database error", e);
            } finally {
                returnConnectionPool(pool);
            }
            
            return null;
        });
    }

    public CompletableFuture<Integer> createUsersAsync(int count) {
        return CompletableFuture.supplyAsync(() -> {
            try (Connection connection = DriverManager.getConnection(getConnectionString())) {
                // Clear existing users
                try (Statement statement = connection.createStatement()) {
                    statement.execute("DELETE FROM user");
                }

                // Insert new users in transaction
                connection.setAutoCommit(false);
                
                String insertSql = "INSERT INTO user (mail, hashed_password) VALUES (?, ?)";
                try (PreparedStatement statement = connection.prepareStatement(insertSql)) {
                    int inserted = 0;
                    for (int i = 1; i <= count; i++) {
                        String email = String.format("user%d@example.com", i);
                        String password = String.format("password%d", i);
                        String hashedPassword = computeSha256Hash(password);

                        statement.setString(1, email);
                        statement.setString(2, hashedPassword);
                        statement.addBatch();
                        inserted++;

                        // Execute batch every 1000 records
                        if (i % 1000 == 0) {
                            statement.executeBatch();
                        }
                    }
                    
                    // Execute remaining batch
                    statement.executeBatch();
                    connection.commit();
                    return inserted;
                }
            } catch (SQLException e) {
                throw new RuntimeException("Failed to create users", e);
            }
        });
    }

    private String computeSha256Hash(String rawData) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] hash = digest.digest(rawData.getBytes());
            StringBuilder hexString = new StringBuilder();
            
            for (byte b : hash) {
                String hex = Integer.toHexString(0xff & b);
                if (hex.length() == 1) {
                    hexString.append('0');
                }
                hexString.append(hex);
            }
            
            return hexString.toString();
        } catch (NoSuchAlgorithmException e) {
            throw new RuntimeException("SHA-256 algorithm not available", e);
        }
    }

    @PreDestroy
    public void cleanup() {
        // Close all pooled connections when service is destroyed
        ConnectionPool pool;
        while ((pool = connectionPools.poll()) != null) {
            pool.close();
        }
        System.out.println("Closed all connection pools. Total closed: " + poolSize.get());
    }
}
