#!/bin/bash
# Build and run Java app with GraalVM Dockerfile
docker build -f Dockerfile.GraalVM -t java-graalvm .
docker run --rm -it --name java-graalvm -p 6000:6000 java-graalvm
