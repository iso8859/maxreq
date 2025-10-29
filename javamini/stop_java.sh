#!/bin/bash
# Stop all Java Docker containers by name
docker stop javamini-graalvm 2>/dev/null || echo "javamini-graalvm not running."
docker stop javamini-azul 2>/dev/null || echo "javamini-azul not running."
docker stop javamini-openj9 2>/dev/null || echo "javamini-openj9 not running."
