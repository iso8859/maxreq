const axios = require('axios');

class ApiPerformanceTester {
    constructor() {
        this.dotnetUrl = 'http://localhost:5150';
        this.nodeUrl = 'http://localhost:3000';
        this.testData = {
            username: 'user1@example.com',
            hashedPassword: '0b14d501a594442a01c6859541bcb3e8164d183d32937b851835442f69d5c94e'
        };
    }

    async measureRequestTime(url, data) {
        const start = Date.now();
        try {
            const response = await axios.post(url, data, {
                headers: { 'Content-Type': 'application/json' },
                timeout: 5000
            });
            const end = Date.now();
            return {
                success: true,
                responseTime: end - start,
                status: response.status,
                data: response.data
            };
        } catch (error) {
            const end = Date.now();
            return {
                success: false,
                responseTime: end - start,
                error: error.message
            };
        }
    }

    async runSingleTest(apiName, url) {
        console.log(`\n🔍 Testing ${apiName}...`);
        const result = await this.measureRequestTime(url, this.testData);
        
        if (result.success) {
            console.log(`✅ ${apiName}: ${result.responseTime}ms (Status: ${result.status})`);
            console.log(`   Response: ${JSON.stringify(result.data)}`);
        } else {
            console.log(`❌ ${apiName}: ${result.responseTime}ms (Error: ${result.error})`);
        }
        
        return result;
    }

    async runLoadTest(apiName, url, requestCount = 10) {
        console.log(`\n⚡ Load Testing ${apiName} (${requestCount} requests)...`);
        
        const promises = [];
        const startTime = Date.now();
        
        for (let i = 0; i < requestCount; i++) {
            promises.push(this.measureRequestTime(url, this.testData));
        }
        
        const results = await Promise.all(promises);
        const endTime = Date.now();
        
        const successful = results.filter(r => r.success);
        const failed = results.filter(r => !r.success);
        const responseTimes = successful.map(r => r.responseTime);
        
        const stats = {
            totalTime: endTime - startTime,
            totalRequests: requestCount,
            successful: successful.length,
            failed: failed.length,
            avgResponseTime: responseTimes.length > 0 ? 
                Math.round(responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length) : 0,
            minResponseTime: responseTimes.length > 0 ? Math.min(...responseTimes) : 0,
            maxResponseTime: responseTimes.length > 0 ? Math.max(...responseTimes) : 0,
            requestsPerSecond: Math.round((requestCount / (endTime - startTime)) * 1000)
        };
        
        console.log(`📊 ${apiName} Load Test Results:`);
        console.log(`   Total Time: ${stats.totalTime}ms`);
        console.log(`   Requests: ${stats.successful}/${stats.totalRequests} successful`);
        console.log(`   Avg Response Time: ${stats.avgResponseTime}ms`);
        console.log(`   Min/Max Response Time: ${stats.minResponseTime}ms / ${stats.maxResponseTime}ms`);
        console.log(`   Requests/Second: ${stats.requestsPerSecond}`);
        
        return stats;
    }

    async compareApis() {
        console.log('🏁 API Performance Comparison');
        console.log('=' .repeat(50));
        
        // Single request tests
        const dotnetResult = await this.runSingleTest('.NET API', `${this.dotnetUrl}/api/auth/get-user-token`);
        const nodeResult = await this.runSingleTest('Node.js API', `${this.nodeUrl}/api/auth/get-user-token`);
        
        // Load tests
        const loadTestCount = 50;
        const dotnetLoadResult = await this.runLoadTest('.NET API', `${this.dotnetUrl}/api/auth/get-user-token`, loadTestCount);
        const nodeLoadResult = await this.runLoadTest('Node.js API', `${this.nodeUrl}/api/auth/get-user-token`, loadTestCount);
        
        // Comparison summary
        console.log('\n🎯 Performance Summary:');
        console.log('=' .repeat(50));
        
        if (dotnetResult.success && nodeResult.success) {
            console.log(`Single Request:`);
            console.log(`  .NET: ${dotnetResult.responseTime}ms`);
            console.log(`  Node.js: ${nodeResult.responseTime}ms`);
            console.log(`  Winner: ${dotnetResult.responseTime < nodeResult.responseTime ? '.NET' : 'Node.js'}`);
        }
        
        console.log(`\nLoad Test (${loadTestCount} requests):`);
        console.log(`  .NET: ${dotnetLoadResult.avgResponseTime}ms avg, ${dotnetLoadResult.requestsPerSecond} req/s`);
        console.log(`  Node.js: ${nodeLoadResult.avgResponseTime}ms avg, ${nodeLoadResult.requestsPerSecond} req/s`);
        console.log(`  Winner: ${dotnetLoadResult.requestsPerSecond > nodeLoadResult.requestsPerSecond ? '.NET' : 'Node.js'}`);
    }
}

// Run the performance tests
async function main() {
    const tester = new ApiPerformanceTester();
    
    try {
        await tester.compareApis();
    } catch (error) {
        console.error('Performance test failed:', error.message);
        console.log('\n💡 Make sure both APIs are running:');
        console.log('   .NET API: http://localhost:5150');
        console.log('   Node.js API: http://localhost:3000');
    }
}

if (require.main === module) {
    main();
}

module.exports = ApiPerformanceTester;
