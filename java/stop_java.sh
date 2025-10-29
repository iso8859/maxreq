#!/bin/bash
# Stop all Java Docker containers by name
docker stop java-graalvm 2>/dev/null || echo "java-graalvm not running."
docker stop java-azul 2>/dev/null || echo "java-azul not running."
docker stop java-openj9 2>/dev/null || echo "java-openj9 not running."
