namespace UserTokenApi.DTOs
{
    public record LoginRequest
    {
        public string Username { get; set; } = string.Empty;
        public string HashedPassword { get; set; } = string.Empty;
    }

    public record LoginResponse
    {
        public bool Success { get; set; }
        public long? UserId { get; set; }
        public string? ErrorMessage { get; set; }
    }
}
