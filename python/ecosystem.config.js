const path = require('path');

const INSTANCES = 16;
const START_PORT = 3001;
const CWD = __dirname; // Assumes this file is in the 'python' directory

const apps = [];

for (let i = 0; i < INSTANCES; i++) {
  const port = START_PORT + i;
  apps.push({
    // --- General ---
    name: `python-api-${port}`,
    script: 'uvicorn',
    args: `main:app --host 0.0.0.0 --port ${port}`,
    cwd: CWD,
    interpreter: 'python',
    env: {
      PYTHONENV: 'production'
    },

    // --- Restart Strategy ---
    // Enable auto-restart (this is true by default)
    autorestart: true,
    // Delay between restarts (in ms) to prevent hammering
    restart_delay: 5000,
    // Minimum uptime to be considered a stable restart (e.g., '30s')
    min_uptime: '30s',
    // Limit the number of unstable restarts
    max_restarts: 10,
  });
}

module.exports = {
  apps: apps
};
