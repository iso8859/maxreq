using Microsoft.AspNetCore.Mvc;
using UserTokenApi.DTOs;
using UserTokenApi.Services;

namespace UserTokenApi.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class AuthController : ControllerBase
    {
        private readonly DatabaseService _databaseService;
        private readonly ILogger<AuthController> _logger;

        public AuthController(DatabaseService databaseService, ILogger<AuthController> logger)
        {
            _databaseService = databaseService;
            _logger = logger;
        }

        [HttpPost("get-user-token")]
        public async Task<ActionResult<LoginResponse>> GetUserToken([FromBody] LoginRequest request)
        {
            var user = await _databaseService.GetUserByMailAndPasswordAsync(request.Username, request.HashedPassword);

            if (user == null)
                return Ok(new LoginResponse
                {
                    Success = false,
                    ErrorMessage = "Invalid username or password"
                });

            return Ok(new LoginResponse
            {
                Success = true,
                UserId = user.Id
            });
        }

        [HttpGet("create-db")]
        public async Task<ActionResult<string>> CreateDatabase()
        {
            try
            {
                var count = await _databaseService.CreateUsersAsync(10000);
                return Ok($"Successfully created {count} users in the database");
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error creating database");
                return StatusCode(500, "An error occurred while creating the database");
            }
        }
    }
}