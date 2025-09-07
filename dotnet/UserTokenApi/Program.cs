using UserTokenApi.Services;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddControllers();
builder.Services.AddSingleton<DatabaseService>();

var app = builder.Build();

// Initialize database on startup
using (var scope = app.Services.CreateScope())
{
    var databaseService = scope.ServiceProvider.GetRequiredService<DatabaseService>();
    await databaseService.InitializeDatabaseAsync();
}

app.MapControllers();

app.Run();
