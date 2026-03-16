cmake_minimum_required(VERSION 3.10)
project(drogon_api)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_BUILD_TYPE Release)

# Find Drogon (it will find dependencies)
find_package(Drogon REQUIRED)

add_executable(drogon-api cpp.cpp)

target_link_libraries(drogon-api PRIVATE Drogon::Drogon)

target_compile_options(drogon-api PRIVATE -O3 -march=native)
