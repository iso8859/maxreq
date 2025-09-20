using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Logging;

var app = WebApplication.Create(args);
app.Urls.Add("http://*:5000");
DatabaseService _databaseService = new DatabaseService(app.Configuration);
await _databaseService.InitializeDatabaseAsync();

var api = app.MapGroup("/api/auth");

api.MapPost("/get-user-token", async ([FromBody] LoginRequest request, ILogger<Program> logger) =>
{
    try
    {
        var user = await _databaseService.GetUserByMailAndPasswordAsync(request);
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
        logger.LogError(ex, "Error during user authentication");
        return Results.Ok(new LoginResponse
        {
            Success = false,
            ErrorMessage = "An error occurred during authentication"
        });
    }
});

api.MapGet("/create-db", async (ILogger<Program> logger) =>
{
    try
    {
        var count = await _databaseService.CreateUsersAsync(10000);
        return Results.Ok($"Successfully created {count} users in the database");
    }
    catch (Exception ex)
    {
        logger.LogError(ex, "Error creating database");
        return Results.Problem("An error occurred while creating the database", statusCode: 500);
    }
});

Console.WriteLine("  GET  /api/auth/health");
Console.WriteLine("  POST /api/auth/get-user-token");
Console.WriteLine("  GET  /api/auth/create-db");

app.Run();
