This directory contains a simple load testing client for the UserToken API implemented in C# .NET 9.

To build it

```bash
dotnet build -c Release ApiLoadTester.csproj
```

To run the load tester:

```bash
bin\Release\net9.0\ApiLoadTester.exe [url] [totalRequests] [concurrentClients] [no_db]
```

For example:

```bash
.\bin\Release\net9.0\ApiLoadTester.exe 10000 16 python
```