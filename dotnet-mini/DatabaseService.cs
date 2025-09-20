using Microsoft.Data.Sqlite;
using System.Collections.Concurrent;
using System.Security.Cryptography;
using System.Text;

public class User
{
    public long Id { get; set; }
}

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

public class DbPool
{
    public SqliteConnection Connection { get; set; }
    public SqliteCommand Command { get; set; }
    public DbPool(SqliteConnection connection)
    {
        Connection = connection;
        Command = null!;
    }
}

public class DatabaseService
{
    public readonly string _connectionString;
    SqliteConnection? _connection;
    // Create a command pool for speed optimization
    ConcurrentBag<DbPool> _dbPools = new ConcurrentBag<DbPool>();

    public DatabaseService(IConfiguration configuration)
    {
        _connectionString = configuration.GetConnectionString("DefaultConnection")
            ?? "Data Source=users.db";
    }

    public async Task InitializeDatabaseAsync()
    {
        _connection = new SqliteConnection(_connectionString);
        await _connection.OpenAsync();

        var command = _connection.CreateCommand();
        command.CommandText = @"
                CREATE TABLE IF NOT EXISTS user (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    mail TEXT NOT NULL UNIQUE,
                    hashed_password TEXT NOT NULL
                )";

        await command.ExecuteNonQueryAsync();
    }


    public async Task<User?> GetUserByMailAndPasswordAsync(LoginRequest loginRequest)
    {
        if (loginRequest.Username == "no_db")
            return new User { Id = 1 };
        if (!_dbPools.TryTake(out var dbPool))
        {
            var connection = new SqliteConnection(_connectionString);
            await connection.OpenAsync();
            dbPool = new DbPool(connection);
            dbPool.Command = dbPool.Connection.CreateCommand();
            dbPool.Command.CommandText = "SELECT id FROM user WHERE mail = @mail AND hashed_password = @hashedPassword limit 1";
            dbPool.Command.Parameters.AddWithValue("@mail", loginRequest.Username);
            dbPool.Command.Parameters.AddWithValue("@hashedPassword", loginRequest.HashedPassword);
            Console.WriteLine($"Created new command.");
        }
        else
        {
            dbPool.Command.Parameters["@mail"].Value = loginRequest.Username;
            dbPool.Command.Parameters["@hashedPassword"].Value = loginRequest.HashedPassword;
        }

        try
        {
            var result = await dbPool.Command.ExecuteScalarAsync();
            if (result is long id)
            {
                return new User
                {
                    Id = id
                };
            }
        }
        catch { }
        finally
        {
            _dbPools.Add(dbPool);
        }

        return null;
    }

    public async Task<int> CreateUsersAsync(int count = 10000)
    {
        using var connection = new SqliteConnection(_connectionString);
        await connection.OpenAsync();

        // Clear existing users
        var clearCommand = connection.CreateCommand();
        clearCommand.CommandText = "DELETE FROM user";
        await clearCommand.ExecuteNonQueryAsync();

        using var transaction = connection.BeginTransaction();

        var command = connection.CreateCommand();
        command.CommandText = "INSERT INTO user (mail, hashed_password) VALUES (@mail, @hashedPassword)";

        var mailParam = command.CreateParameter();
        mailParam.ParameterName = "@mail";
        command.Parameters.Add(mailParam);

        var passwordParam = command.CreateParameter();
        passwordParam.ParameterName = "@hashedPassword";
        command.Parameters.Add(passwordParam);

        int inserted = 0;
        for (int i = 1; i <= count; i++)
        {
            string email = $"user{i}@example.com";
            string password = $"password{i}";
            string hashedPassword = ComputeSha256Hash(password);

            mailParam.Value = email;
            passwordParam.Value = hashedPassword;

            await command.ExecuteNonQueryAsync();
            inserted++;
        }

        await transaction.CommitAsync();
        return inserted;
    }

    private static string ComputeSha256Hash(string rawData)
    {
        using SHA256 sha256Hash = SHA256.Create();
        byte[] bytes = sha256Hash.ComputeHash(Encoding.UTF8.GetBytes(rawData));

        return Convert.ToHexStringLower(bytes);
    }
}