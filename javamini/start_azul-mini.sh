#!/bin/bash
# Build and run Java app with Azul Dockerfile
docker build -f Dockerfile.Azul -t javamini-azul .
docker run --rm -it --name javamini-azul -p 6000:6000 javamini-azul
