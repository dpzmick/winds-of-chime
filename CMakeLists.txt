cmake_minimum_required(VERSION 2.6)
project(winds-of-chime)
find_package(PkgConfig REQUIRED)

set(SHADERS
    ${CMAKE_SOURCE_DIR}/shaders/shader.vert
    ${CMAKE_SOURCE_DIR}/shaders/shader.frag)

foreach(SHADER ${SHADERS})
    get_filename_component(BASENAME ${SHADER} NAME)
    set(SPIRV "${PROJECT_BINARY_DIR}/shaders/${BASENAME}.spv")
    add_custom_command(
        OUTPUT ${SPIRV}
        COMMAND ${CMAKE_COMMAND} -E make_directory "${PROJECT_BINARY_DIR}/shaders"
        COMMAND glslc ${SHADER} -o ${SPIRV}
        COMMENT "Building ${SPIRV}"
        DEPENDS ${SHADER})
    list(APPEND SPIRV_SHADERS ${SPIRV})
endforeach()

add_custom_target(Shaders DEPENDS ${SPIRV_SHADERS})

add_executable(winds-of-chime main.c)
add_dependencies(winds-of-chime Shaders)

# find_package(SDL REQUIRED)
# target_link_libraries(winds-of-chime ${SDL_LIBRARY})

find_package(Vulkan REQUIRED)
target_link_libraries(winds-of-chime Vulkan::Vulkan)

pkg_check_modules(JACK REQUIRED jack)
target_link_libraries(winds-of-chime jack)

find_package(glfw3 REQUIRED)
target_link_libraries(winds-of-chime glfw)
