#!/bin/bash
# Script to stop and remove the Bun Docker container

docker stop bun-app 2>/dev/null || echo "bun-app container is not running."
