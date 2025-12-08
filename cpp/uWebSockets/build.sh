#!/bin/bash
g++ -std=c++17 -O2 maxreq.cpp -o maxreq -I/usr/include -lsqlite3 -lssl -lcrypto -lpthread