cmake_minimum_required(VERSION 3.17)
project(winds-of-chime)

string(APPEND CMAKE_C_FLAGS " -Wall -Wextra -Werror -fverbose-asm -g")
string(APPEND CMAKE_C_FLAGS " -Wno-unused -Wno-unused-parameter -Wconversion")
string(APPEND CMAKE_C_FLAGS " -std=c11")
string(APPEND CMAKE_C_FLAGS " -fvisibility=hidden -fstrict-aliasing")
string(APPEND CMAKE_C_FLAGS " -O3 -march=native -mtune=native")
string(APPEND CMAKE_C_FLAGS " -flto")

add_subdirectory(third-party)
add_subdirectory(src)
