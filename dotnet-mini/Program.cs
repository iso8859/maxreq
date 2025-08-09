using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();

DatabaseService _databaseService = new DatabaseService(builder.Configuration);
ILogger<Program> _logger = app.Services.GetRequiredService<ILogger<Program>>();

app.MapPost("api/auth/get-user-token", async ([FromBody] LoginRequest request) =>
{
    try
    {
        if (string.IsNullOrEmpty(request.Username) || string.IsNullOrEmpty(request.HashedPassword))
        {
            return Results.Ok(new LoginResponse
            {
                Success = false,
                ErrorMessage = "Username and hashed password are required"
            });
        }

        var user = await _databaseService.GetUserByMailAndPasswordAsync(request.Username, request.HashedPassword);

        if (user != null)
        {
            return Results.Ok(new LoginResponse
            {
                Success = true,
                UserId = user.Id
            });
        }
        else
        {
            return Results.Ok(new LoginResponse
            {
                Success = false,
                ErrorMessage = "Invalid username or password"
            });
        }
    }
    catch (Exception ex)
    {
        _logger.LogError(ex, "Error during user authentication");
        return Results.Ok(new LoginResponse
        {
            Success = false,
            ErrorMessage = "An error occurred during authentication"
        });
    }
});

app.MapGet("/api/auth/create-db", async () =>
{
            try
            {
                var count = await _databaseService.CreateUsersAsync(10000);
                return Results.Ok($"Successfully created {count} users in the database");
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error creating database");
                return Results.Problem("An error occurred while creating the database", statusCode: 500);
            }
});

app.Run();
