#!/bin/bash

# Install dependencies
echo "Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y cmake g++ libjsoncpp-dev libsqlite3-dev libssl-dev uuid-dev zlib1g-dev

# Install Drogon
if ! pkg-config --exists drogon; then
    echo "Installing Drogon from source..."
    cd /tmp
    git clone https://github.com/drogonframework/drogon
    cd drogon
    git submodule update --init
    mkdir build
    cd build
    cmake ..
    make -j$(nproc)
    sudo make install
    sudo ldconfig
fi

cd /home/debian/maxreq/cpp/drogon

# Build the application
echo "Building application..."
mkdir -p build
cd build
cmake ..
make -j$(nproc)

echo "Build complete! Run with: ./drogon-api"
