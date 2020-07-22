add_subdirectory(shaders)

add_subdirectory(util)
#add_subdirectory(pup)
add_subdirectory(tracing)
#add_subdirectory(test)

set(PUP_BUILD_DIR ${CMAKE_CURRENT_BINARY_DIR}/pup/_build/)
set(C_EXE ${PUP_BUILD_DIR}/default/genCTracingStructs.exe)
set(PY_EXE ${PUP_BUILD_DIR}/default/genPyTracingStructs.exe)

include(ExternalProject)
ExternalProject_Add(pup
  SOURCE_DIR pup
  BINARY_DIR ${CMAKE_CURRENT_SOURCE_DIR}/pup
  BUILD_COMMAND dune build --build-dir=${PUP_BUILD_DIR}
  CONFIGURE_COMMAND ""
  INSTALL_COMMAND ""
  BUILD_ALWAYS yes
  BUILD_BYPRODUCTS ${PY_EXE} ${C_EXE})

add_custom_command(
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h
  DEPENDS ${C_EXE}
  COMMAND ${C_EXE} > ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h)
add_custom_target(generate-c-tracing-structs DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h)
add_dependencies(generate-c-tracing-structs pup)

add_custom_command(
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.py
  DEPENDS ${PY_EXE}
  COMMAND ${PY_EXE} > ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.py)
add_custom_target(generate-py-tracing-structs DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.py)
add_dependencies(generate-py-tracing-structs pup)

# finally, turn this into a library so that it can be used downstream (and configure header include path)

add_library(c-tracing-structs INTERFACE)
target_include_directories(c-tracing-structs INTERFACE ${CMAKE_CURRENT_BINARY_DIR})
add_dependencies(c-tracing-structs generate-c-tracing-structs)

add_executable(main common.c app.c main.c)
target_link_libraries(main util volk glfw Threads::Threads c-tracing-structs tracing)
