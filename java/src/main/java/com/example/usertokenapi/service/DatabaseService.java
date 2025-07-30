package com.example.usertokenapi.service;

import com.example.usertokenapi.model.User;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import jakarta.annotation.PostConstruct;
import java.sql.*;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.concurrent.CompletableFuture;

@Service
public class DatabaseService {

    @Value("${app.database.path:users.db}")
    private String databasePath;

    private String getConnectionString() {
        return "jdbc:sqlite:" + databasePath;
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
            try (Connection connection = DriverManager.getConnection(getConnectionString())) {
                String sql = "SELECT id FROM user WHERE mail = ? AND hashed_password = ?";
                
                try (PreparedStatement statement = connection.prepareStatement(sql)) {
                    statement.setString(1, mail);
                    statement.setString(2, hashedPassword);
                    
                    try (ResultSet resultSet = statement.executeQuery()) {
                        if (resultSet.next()) {
                            User user = new User();
                            user.setId(resultSet.getLong("id"));
                            return user;
                        }
                    }
                }
            } catch (SQLException e) {
                throw new RuntimeException("Database error", e);
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
}
