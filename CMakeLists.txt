cmake_minimum_required(VERSION 2.6)
project(winds-of-chime)
find_package(PkgConfig REQUIRED)

add_executable(winds-of-chime main.c)

# find_package(SDL REQUIRED)
# target_link_libraries(winds-of-chime ${SDL_LIBRARY})

find_package(Vulkan REQUIRED)
target_link_libraries(winds-of-chime Vulkan::Vulkan)

pkg_check_modules(JACK REQUIRED jack)
target_link_libraries(winds-of-chime jack)

find_package(glfw3 REQUIRED)
target_link_libraries(winds-of-chime glfw)
