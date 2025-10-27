#!/bin/bash
# Script to build and run Bun app in Docker

# Build the Docker image
docker build -t bun-app .

# Run the Docker container, mapping port 8084 (change if needed)
docker run --rm -it -p 8084:8084 --name bun-app bun-app
