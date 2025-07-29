namespace UserTokenApi.DTOs
{
    public class LoginRequest
    {
        public string Username { get; set; } = string.Empty;
        public string HashedPassword { get; set; } = string.Empty;
    }

    public class LoginResponse
    {
        public bool Success { get; set; }
        public long? UserId { get; set; }
        public string? ErrorMessage { get; set; }
    }
}
