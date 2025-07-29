const crypto = require('crypto');

// Function to generate SHA256 hash
function generateHash(password) {
    return crypto.createHash('sha256').update(password).digest('hex');
}

// Generate hashes for common test passwords
const testPasswords = ['password1', 'password100', 'password1000', 'password10000'];

console.log('Password Hash Generator');
console.log('======================');

testPasswords.forEach(password => {
    const hash = generateHash(password);
    console.log(`Password: ${password}`);
    console.log(`Hash:     ${hash}`);
    console.log('');
});

// If password provided as command line argument
if (process.argv[2]) {
    const customPassword = process.argv[2];
    console.log(`Custom Password: ${customPassword}`);
    console.log(`Hash:           ${generateHash(customPassword)}`);
}
