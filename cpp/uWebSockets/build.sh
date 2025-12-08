#!/bin/bash
gcc -std=c11 -O2 -c sqlite/sqlite3.c -o sqlite3.o
g++ -std=c++17 -fpermissive -O2 @files.txt -o maxreq -DLIBUS_NO_SSL -DLIBUS_USE_OPENSSL -DLIBUS_USE_LIBUV -I/usr/include -IuSockets -lsqlite3 -lssl -lcrypto -lpthread -lz -luv sqlite3.o