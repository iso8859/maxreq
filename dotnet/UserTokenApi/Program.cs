using UserTokenApi.Services;
using System.Text.Json;
using System.Text.Json.Serialization;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddControllers();
DatabaseService _databaseService = new DatabaseService(builder.Configuration);
await _databaseService.InitializeDatabaseAsync();
builder.Services.AddSingleton(_databaseService);

var app = builder.Build();

app.MapControllers();

app.Run();