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
