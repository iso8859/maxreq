# Multi Dev Language API Performance Test

In this repository you will find simple implementations of a UserToken API in multiple programming languages.

The objective is to get the fastest response for each technology.

We can get better performance by optimizing the code, or choosing the right runtime. 

For Java minimal API is faster than Spring Boot.

For Typescript or Javascript, Bun is faster than Node.js. Bun sqlite implementation is really optimized.

# Test client

A test client is provided in C# .NET 9 in directory dotnet_test. Look at readme.md in this directory for instructions.

# Check if everything is working

To be sure api calls are real I added some trace in the console that display userId to be sure we get a real userId from the database.

# You want to improve ?

This project is open to any improvement, please fork and create a pull request.

You should keep the same API, same database (sqlite), same data (10000 users) and same test procedure (100000 requests, 16 concurrent clients).

Please test on your machine the current version and compare your improvement. Please commmit only if you get really better performance.

Result below is from my machine under Windows 10, AMD Ryzen 7 2700X, 32GB RAM, SSD NVMe.

I keep the best result after 5 runs.

On Windows 10
![Performance Comparison Chart Windows](illustration.png)

On Debian 6.1
![Performance Comparison Chart Debian](illustration-debian.png)

# Conclusion

The Debian tests are the most interresting because we do not put production on Windows 10. 

**We can observe a real difference between the two platforms.**
Results are more expected on Debian.

I didn't tried C++ on Debian because C++ is too complex today to start a new project and it's very tricky and time consuming to obtain good results.

# License

This project is for educational and benchmarking purposes.

