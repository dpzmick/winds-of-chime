cmake_minimum_required(VERSION 3.17)
project(winds-of-chime)


string(APPEND CMAKE_C_FLAGS " -Wall -Wextra -Werror -fverbose-asm -g")
string(APPEND CMAKE_C_FLAGS " -Wno-unused -Wno-unused-parameter -Wconversion")
string(APPEND CMAKE_C_FLAGS " -std=c11")
string(APPEND CMAKE_C_FLAGS " -fvisibility=hidden -fstrict-aliasing")
string(APPEND CMAKE_C_FLAGS " -O3 -march=native -mtune=native")
string(APPEND CMAKE_C_FLAGS " -flto")
string(APPEND CMAKE_C_FLAGS " -D_GNU_SOURCE")

# hack to print __FILE__ paths relative to project root
# cmake uses absolute paths for everything
# seems fragile
string(LENGTH "${CMAKE_SOURCE_DIR}/" SOURCE_PATH_SIZE)
string(APPEND CMAKE_C_FLAGS " -D__FILE_HEADER_LEN__=${SOURCE_PATH_SIZE}")

find_package(glfw3 REQUIRED)

set(VOLK_STATIC_DEFINES VK_USE_PLATFORM_XLIB_KHR)
add_subdirectory(third-party)
add_subdirectory(src)
