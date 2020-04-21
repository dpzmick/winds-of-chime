set(shader_src ${CMAKE_CURRENT_SOURCE_DIR}/shader.glsl)
set(shader_dst ${CMAKE_CURRENT_BINARY_DIR}/shader.spv)
message("shader in ${shader_src}")
add_custom_command(
  OUTPUT  ${shader_dst}
  DEPENDS ${shader_src}
  COMMAND glslc -fshader-stage=compute ${shader_src} -o ${shader_dst}
)
add_custom_target(shader ALL DEPENDS ${shader_dst})

add_executable(main common.c log.c app.c main.c)
target_link_libraries(main volk glfw)

add_executable(test-glfw test-glfw.c)
target_link_libraries(test-glfw vulkan glfw)
