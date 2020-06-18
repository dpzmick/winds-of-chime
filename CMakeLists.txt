cmake_minimum_required(VERSION 3.17)
project(winds-of-chime)


string(APPEND CMAKE_C_FLAGS " -Wall -Wextra -Werror -fverbose-asm -g")
string(APPEND CMAKE_C_FLAGS " -Wno-unused -Wno-unused-parameter -Wconversion")
string(APPEND CMAKE_C_FLAGS " -std=c11")
string(APPEND CMAKE_C_FLAGS " -fvisibility=hidden -fstrict-aliasing")
#string(APPEND CMAKE_C_FLAGS " -O3 -march=native -mtune=native -flto")
string(APPEND CMAKE_C_FLAGS " -O0")
string(APPEND CMAKE_C_FLAGS " -D_GNU_SOURCE")
string(APPEND CMAKE_C_FLAGS " -fsanitize=address")

find_package(glfw3 REQUIRED)

set(VOLK_STATIC_DEFINES VK_USE_PLATFORM_XLIB_KHR)
add_subdirectory(third-party)
add_subdirectory(src)
