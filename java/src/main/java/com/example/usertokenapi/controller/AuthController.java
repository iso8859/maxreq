package com.example.usertokenapi.controller;

import com.example.usertokenapi.dto.LoginRequest;
import com.example.usertokenapi.dto.LoginResponse;
import com.example.usertokenapi.model.User;
import com.example.usertokenapi.service.DatabaseService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.concurrent.CompletableFuture;

@RestController
@RequestMapping("/api/auth")
public class AuthController {

    private static final Logger logger = LoggerFactory.getLogger(AuthController.class);

    @Autowired
    private DatabaseService databaseService;

    @PostMapping("/get-user-token")
    public CompletableFuture<ResponseEntity<LoginResponse>> getUserToken(@RequestBody LoginRequest request) {
        try {
            if (request.getUsername() == null || request.getUsername().isEmpty() ||
                request.getHashedPassword() == null || request.getHashedPassword().isEmpty()) {
                return CompletableFuture.completedFuture(
                    ResponseEntity.ok(LoginResponse.failure("Username and hashed password are required"))
                );
            }

            return databaseService.getUserByMailAndPasswordAsync(request.getUsername(), request.getHashedPassword())
                .thenApply(user -> {
                    if (user != null) {
                        return ResponseEntity.ok(LoginResponse.success(user.getId()));
                    } else {
                        return ResponseEntity.ok(LoginResponse.failure("Invalid username or password"));
                    }
                })
                .exceptionally(ex -> {
                    logger.error("Error during user authentication", ex);
                    return ResponseEntity.ok(LoginResponse.failure("An error occurred during authentication"));
                });
        } catch (Exception ex) {
            logger.error("Error during user authentication", ex);
            return CompletableFuture.completedFuture(
                ResponseEntity.ok(LoginResponse.failure("An error occurred during authentication"))
            );
        }
    }

    @GetMapping("/create-db")
    public CompletableFuture<ResponseEntity<String>> createDatabase() {
        try {
            return databaseService.createUsersAsync(10000)
                .thenApply(count -> ResponseEntity.ok(String.format("Successfully created %d users in the database", count)))
                .exceptionally(ex -> {
                    logger.error("Error creating database", ex);
                    return ResponseEntity.status(500).body("An error occurred while creating the database");
                });
        } catch (Exception ex) {
            logger.error("Error creating database", ex);
            return CompletableFuture.completedFuture(
                ResponseEntity.status(500).body("An error occurred while creating the database")
            );
        }
    }
}
