# Makefile for building maxreq.cpp with uWebSockets

# Compiler settings
CC = gcc
CXX = g++
CFLAGS = -std=c11 -O2
CXXFLAGS = -std=c++17 -O2 -fpermissive
DEFINES = -DLIBUS_NO_SSL -DLIBUS_USE_OPENSSL -DLIBUS_USE_LIBUV
INCLUDES = -I. -Isrc -IuSockets -Isqlite
LIBS = -lssl -lcrypto -lpthread -lz -luv -ldl
WARNFLAGS = -Wno-deprecated-declarations

# Source files
SQLITE_SRC = sqlite/sqlite3.c
MAIN_SRC = maxreq.cpp
USOCKETS_SRC = $(wildcard uSockets/*.c uSockets/eventing/*.c uSockets/io_uring/*.c)

# Object files
SQLITE_OBJ = sqlite3.o

# Output
TARGET = maxreq

# Default target
all: $(TARGET)

# Compile SQLite separately (as C code)
$(SQLITE_OBJ): $(SQLITE_SRC)
	$(CC) $(CFLAGS) -DSQLITE_THREADSAFE=1 -DSQLITE_ENABLE_FTS5 -c $(SQLITE_SRC) -o $(SQLITE_OBJ)

# Build main executable - MAINTENANT AVEC LES BONNES DÉPENDANCES
$(TARGET): $(SQLITE_OBJ) $(MAIN_SRC) $(USOCKETS_SRC)
	$(CXX) $(CXXFLAGS) $(DEFINES) $(WARNFLAGS) $(MAIN_SRC) $(SQLITE_OBJ) $(USOCKETS_SRC) -o $(TARGET) $(INCLUDES) $(LIBS)
# Clean build artifacts
clean:
	rm -f $(TARGET) $(SQLITE_OBJ) $(USOCKETS_SRC:.c=.o)

# Run the application
run: $(TARGET)
	./$(TARGET)
# Rebuild everything
rebuild: clean all

.PHONY: all clean run rebuild