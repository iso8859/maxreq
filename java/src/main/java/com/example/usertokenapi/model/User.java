package com.example.usertokenapi.model;

public class User {
    private Long id;
    private String mail;
    private String hashedPassword;

    public User() {}

    public User(Long id, String mail, String hashedPassword) {
        this.id = id;
        this.mail = mail;
        this.hashedPassword = hashedPassword;
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public String getMail() {
        return mail;
    }

    public void setMail(String mail) {
        this.mail = mail;
    }

    public String getHashedPassword() {
        return hashedPassword;
    }

    public void setHashedPassword(String hashedPassword) {
        this.hashedPassword = hashedPassword;
    }
}
