namespace UserTokenApi.DTOs
{
    public struct LoginRequest
    {
        public string Username { get; set; } 
        public string HashedPassword { get; set; }
    }

    public struct LoginResponse
    {
        public bool Success { get; set; }
        public long? UserId { get; set; }
        public string? ErrorMessage { get; set; }
    }
}
