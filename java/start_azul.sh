#!/bin/bash
# Build and run Java app with Azul Dockerfile
docker build -f Dockerfile.Azul -t java-azul .
docker run --rm -it --name java-azul -p 6000:6000 java-azul
