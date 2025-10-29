#!/bin/bash
# Build and run Java app with GraalVM Dockerfile
docker build -f Dockerfile.GraalVM-mini -t javamini-graalvm .
docker run --rm -it --name javamini-graalvm -p 6000:6000 javamini-graalvm
