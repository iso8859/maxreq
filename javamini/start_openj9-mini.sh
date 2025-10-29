#!/bin/bash
# Build and run Java app with OpenJ9 Dockerfile
docker build -f Dockerfile.OpenJ9 -t javamini-openj9 .
docker run --rm -it --name javamini-openj9 -p 6000:6000 javamini-openj9
