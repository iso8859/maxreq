#!/bin/bash
# Build and run Java app with OpenJ9 Dockerfile
docker build -f Dockerfile.OpenJ9 -t java-openj9 .
docker run --rm -it --name java-openj9 -p 6000:6000 java-openj9
