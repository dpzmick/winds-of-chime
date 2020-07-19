#add_subdirectory(util)
#add_subdirectory(test)

add_custom_target(shader ALL)
function(add_shader name shader_src shader_dst stage)
  set(in ${CMAKE_CURRENT_SOURCE_DIR}/${shader_src})
  set(out ${CMAKE_CURRENT_BINARY_DIR}/${shader_dst})
  add_custom_command(
    OUTPUT  ${shader_dst}
    DEPENDS ${in}
    COMMAND glslc -fshader-stage=${stage} ${in} -o ${out})
  add_custom_target(shader-${name} DEPENDS ${out})
  add_dependencies(shader shader-${name})
endfunction()

add_shader(vert shader.vert vert.spv vertex)
add_shader(frag shader.frag frag.spv fragment)

#add_executable(simple simple.c rb/pup.c)

#add_executable(unit_test unit_test.c test_hashtable.c)

#add_executable(test_rb_log test_rb_log.c rb_log.c)
#add_executable(test_packer test_rb_packer.c)
#add_executable(test_hashtable test_hashtable.c util/hashtable.c)

#add_executable(test_test util/tests.c util/test.c)

add_executable(main common.c log.c app.c main.c ../third-party/tracy/TracyClient.cpp tracy.cpp)
target_link_libraries(main volk glfw Threads::Threads)

#add_executable(test-glfw test-glfw.c)
#target_link_libraries(test-glfw vulkan glfw)
