using Microsoft.Data.Sqlite;
using UserTokenApi.Models;
using System.Security.Cryptography;
using System.Text;

namespace UserTokenApi.Services
{
    public class DatabaseService
    {
        private readonly string _connectionString;

        public DatabaseService(IConfiguration configuration)
        {
            _connectionString = configuration.GetConnectionString("DefaultConnection")
                                ?? "Data Source=users.db";
        }

        public async Task InitializeDatabaseAsync()
        {
            using var connection = new SqliteConnection(_connectionString);
            await connection.OpenAsync();

            var command = connection.CreateCommand();
            command.CommandText = @"
                CREATE TABLE IF NOT EXISTS user (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    mail TEXT NOT NULL UNIQUE,
                    hashed_password TEXT NOT NULL
                )";

            await command.ExecuteNonQueryAsync();
        }

        public async Task<User?> GetUserByMailAndPasswordAsync(string mail, string hashedPassword)
        {
            using var connection = new SqliteConnection(_connectionString);
            await connection.OpenAsync();

            var command = connection.CreateCommand();
            command.CommandText = "SELECT id FROM user WHERE mail = @mail AND hashed_password = @hashedPassword";
            command.Parameters.AddWithValue("@mail", mail);
            command.Parameters.AddWithValue("@hashedPassword", hashedPassword);

            try
            {
                long id = (long)(await command.ExecuteScalarAsync());
                return new User
                {
                    Id = id
                };
            }
            catch { }

            return null;
        }

        public async Task<int> CreateUsersAsync(int count = 10000)
        {
            await InitializeDatabaseAsync();

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
}