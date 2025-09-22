package com.example.usertokenapi.dto;

import com.fasterxml.jackson.annotation.JsonProperty;

public class LoginResponse {
    @JsonProperty("Success")
    private boolean success;
    
    @JsonProperty("UserId")
    private Long userId;
    
    @JsonProperty("ErrorMessage")
    private String errorMessage;

    public LoginResponse() {}

    public LoginResponse(boolean success, Long userId, String errorMessage) {
        this.success = success;
        this.userId = userId;
        this.errorMessage = errorMessage;
    }

    public static LoginResponse success(Long userId) {
        return new LoginResponse(true, userId, null);
    }

    public static LoginResponse failure(String errorMessage) {
        return new LoginResponse(false, null, errorMessage);
    }

    public boolean isSuccess() {
        return success;
    }

    public void setSuccess(boolean success) {
        this.success = success;
    }

    public Long getUserId() {
        return userId;
    }

    public void setUserId(Long userId) {
        this.userId = userId;
    }

    public String getErrorMessage() {
        return errorMessage;
    }

    public void setErrorMessage(String errorMessage) {
        this.errorMessage = errorMessage;
    }
}
