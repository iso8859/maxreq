import { Database } from "bun:sqlite";
import { createHash } from "crypto";

// DTOs - equivalent to AuthDTOs.cs
interface LoginRequest {
  UserName: string;
  HashedPassword: string;
}

interface LoginResponse {
  Success: boolean;
  UserId?: number;
  ErrorMessage?: string;
}

// Models - equivalent to User.cs
interface User {
  Id: number;
}

// Database service - equivalent to DatabaseService.cs
class DatabaseService {
  private db: Database;
  private connectionString: string;
  private preparedStatements: Map<string, any> = new Map();

  constructor(connectionString: string = "users.db") {
    this.connectionString = connectionString;
    this.db = new Database(connectionString);
    this.initializeDatabase();
  }

  private initializeDatabase(): void {
    // Create table if not exists - equivalent to InitializeDatabaseAsync
    const createTableSQL = `
      CREATE TABLE IF NOT EXISTS user (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        mail TEXT NOT NULL UNIQUE,
        hashed_password TEXT NOT NULL
      )
    `;
    
    this.db.exec(createTableSQL);
    console.log("Database initialized");
  }

  async getUserByMailAndPasswordAsync(loginRequest: LoginRequest): Promise<User | null> {
    // Special case for testing - equivalent to C# version
    if (loginRequest.UserName === "no_db") {
      return { Id: 1 };
    }

    // Get or create prepared statement for performance optimization
    const queryKey = "getUserQuery";
    let stmt = this.preparedStatements.get(queryKey);
    
    if (!stmt) {
      stmt = this.db.prepare("SELECT id FROM user WHERE mail = ?1 AND hashed_password = ?2 LIMIT 1");
      this.preparedStatements.set(queryKey, stmt);
      console.log("Created new prepared statement");
    }

    try {
      //console.log(`Executing query for user: ${loginRequest.UserName} with hashed password: ${loginRequest.HashedPassword}`);
      const result = stmt.get(loginRequest.UserName, loginRequest.HashedPassword) as any;
      //console.log("Result from DB:", result);
      if (result && result.id) {
        return {
          Id: result.id as number
        };
      }
    } catch (error) {
      console.error("Database query error:", error);
    }

    return null;
  }

  async createUsersAsync(count: number = 10000): Promise<number> {
    // Begin transaction for performance
    const transaction = this.db.transaction(() => {
      // Clear existing users
      this.db.exec("DELETE FROM user");

      // Prepare insert statement
      const insertStmt = this.db.prepare("INSERT INTO user (mail, hashed_password) VALUES (?1, ?2)");

      let inserted = 0;
      for (let i = 1; i <= count; i++) {
        const email = `user${i}@example.com`;
        const password = `password${i}`;
        const hashedPassword = this.computeSha256Hash(password);

        try {
          insertStmt.run(email, hashedPassword);
          inserted++;
        } catch (error) {
          console.error(`Failed to insert user ${i}:`, error);
        }
      }

      return inserted;
    });

    return transaction();
  }

  private computeSha256Hash(rawData: string): string {
    // Equivalent to ComputeSha256Hash in C#
    return createHash('sha256').update(rawData, 'utf8').digest('hex');
  }

  close(): void {
    this.db.close();
  }
}

// Auth Controller - equivalent to AuthController.cs
class AuthController {
  private databaseService: DatabaseService;

  constructor(databaseService: DatabaseService) {
    this.databaseService = databaseService;
  }

  // Equivalent to GetUserToken endpoint
  async getUserToken(request: LoginRequest): Promise<LoginResponse> {
    try {
      const user = await this.databaseService.getUserByMailAndPasswordAsync(request);

      if (user === null) {
        return {
          Success: false,
          ErrorMessage: "Invalid username or password"
        };
      }

      return {
        Success: true,
        UserId: user.Id
      };
    } catch (error) {
      console.error("Error during user authentication:", error);
      return {
        Success: false,
        ErrorMessage: "An error occurred during authentication"
      };
    }
  }

  // Equivalent to CreateDatabase endpoint
  async createDatabase(): Promise<{ success: boolean; message: string; statusCode: number }> {
    try {
      const count = await this.databaseService.createUsersAsync(10000);
      return {
        success: true,
        message: `Successfully created ${count} users in the database`,
        statusCode: 200
      };
    } catch (error) {
      console.error("Error creating database:", error);
      return {
        success: false,
        message: "An error occurred while creating the database",
        statusCode: 500
      };
    }
  }
}

// Web server setup using Bun's built-in server
const PORT = process.env.PORT || 8084;
const DB_PATH = process.env.DB_PATH || "users.db";

// Initialize services
const databaseService = new DatabaseService(DB_PATH);
const authController = new AuthController(databaseService);

// Bun server - equivalent to ASP.NET Core setup
const server = Bun.serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);
    const method = req.method;

    // CORS headers
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
      "Content-Type": "application/json"
    };

    // Handle CORS preflight
    if (method === "OPTIONS") {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      // Route: POST /api/auth/get-user-token
      if (method === "POST" && url.pathname === "/api/auth/get-user-token") {
        const body = await req.json() as LoginRequest;
        const response = await authController.getUserToken(body);
        return new Response(JSON.stringify(response), { 
          headers: corsHeaders 
        });
      }

      // Route: GET /api/auth/create-db
      if (method === "GET" && url.pathname === "/api/auth/create-db") {
        const response = await authController.createDatabase();
        return new Response(
          response.success ? response.message : JSON.stringify({ error: response.message }), 
          { 
            status: response.statusCode,
            headers: corsHeaders 
          }
        );
      }

      // Health check endpoint
      if (method === "GET" && url.pathname === "/health") {
        return new Response("UserTokenApi Bun/TypeScript server is running", { 
          headers: corsHeaders 
        });
      }

      // 404 for unknown routes
      return new Response("Not Found", { 
        status: 404,
        headers: corsHeaders 
      });

    } catch (error) {
      console.error("Server error:", error);
      return new Response(JSON.stringify({ error: "Internal server error" }), { 
        status: 500,
        headers: corsHeaders 
      });
    }
  },
});

console.log(`🚀 Bun/TypeScript UserTokenApi server running on http://localhost:${PORT}`);
console.log("Available endpoints:");
console.log(`  POST http://localhost:${PORT}/api/auth/get-user-token - Authenticate user`);
console.log(`  GET  http://localhost:${PORT}/api/auth/create-db - Create test database`);
console.log(`  GET  http://localhost:${PORT}/health - Health check`);

// Graceful shutdown
process.on("SIGINT", () => {
  console.log("\nShutting down server...");
  databaseService.close();
  process.exit(0);
});

process.on("SIGTERM", () => {
  console.log("\nShutting down server...");
  databaseService.close();
  process.exit(0);
});