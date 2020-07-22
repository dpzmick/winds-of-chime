# attempt to compile ocaml with cmake
# might be tricky (ha understatement)

# function(compile_ocaml_file target in out)
#   # ocamlopt doens't seem to always follow instructions about output filename
#   add_custom_command(
#     OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${out}
#     DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${in}
#     COMMAND ocamlopt -c ${CMAKE_CURRENT_SOURCE_DIR}/${in} -o ${CMAKE_CURRENT_BINARY_DIR}/${out} -I ${CMAKE_CURRENT_BINARY_DIR})
#   add_custom_target(${target} DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${out})
# endfunction()

# # ocaml mli files must be compiled first
# # then anything that uses the mli must be compiled into a ".o" equivilent
# #
# # > ocamldep simple.ml
# # simple.cmo : \
# #     pupTypes.cmi \
# #     pupC.cmo \
# #     pup.cmi
# # simple.cmx : \
# #     pupTypes.cmi \
# #     pupC.cmx \
# #     pup.cmx

# compile_ocaml_file(puptypes-cmi pupTypes.mli pupTypes.cmi)
# compile_ocaml_file(pup-cmi pup.mli pup.cmi)

# compile_ocaml_file(pup-cmx pup.ml pup.cmx)
# compile_ocaml_file(pupC-cmx pupC.ml pupC.cmx)
# compile_ocaml_file(pup-c-tracing-structs-gen-cmx c-tracing-structs-gen.ml c-tracing-structs-gen.cmx)

# # build the dep tree
# add_dependencies(pup-cmi puptypes-cmi)
# add_dependencies(pup-cmx pup-cmi puptypes-cmi)
# add_dependencies(pupC-cmx pup-cmi puptypes-cmi)
# add_dependencies(pup-c-tracing-structs-gen-cmx pup-cmx pupC-cmx)   # technically wrong but I don't have an mli for pupC

# add_custom_command(
#   OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/c-tracing-structs-gen

#   DEPENDS
#     ${CMAKE_CURRENT_BINARY_DIR}/pup.cmx
#     ${CMAKE_CURRENT_BINARY_DIR}/pupC.cmx
#     ${CMAKE_CURRENT_BINARY_DIR}/c-tracing-structs-gen.cmx

#   COMMAND ocamlopt -o ${CMAKE_CURRENT_BINARY_DIR}/c-tracing-structs-gen pup.cmx pupC.cmx ${CMAKE_CURRENT_BINARY_DIR}/c-tracing-structs-gen.cmx)

# add_custom_target(pup-c-tracing-structs-gen
#   DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/c-tracing-structs-gen)

# add_dependencies(pup-c-tracing-structs-gen
#   pup-cmx pupC-cmx pup-c-tracing-structs-gen-cmx)

# add_custom_command(
#   OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h
#   DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/c-tracing-structs-gen
#   COMMAND ${CMAKE_CURRENT_BINARY_DIR}/c-tracing-structs-gen > ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h
#   VERBATIM)

# add_custom_target(generate-c-tracing-structs DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h)
# add_dependencies(generate-c-tracing-structs pup-c-tracing-structs-gen)

set(BUILD_DIR ${CMAKE_CURRENT_BINARY_DIR}/_build/)
set(C_EXE ${BUILD_DIR}/default/genCTracingStructs.exe)
set(PY_EXE ${BUILD_DIR}/default/genPyTracingStructs.exe)

include(ExternalProject)
ExternalProject_Add(pup
  SOURCE_DIR pup
  BUILD_COMMAND dune build --build-dir=${BUILD_DIR}
  BUILD_ALWAYS yes
  BUILD_BYPRODUCTS ${PY_EXE} ${C_EXE})

# # add_custom_command(
# #   OUTPUT always_rebuild
# #   COMMAND echo > /dev/null)

# add_custom_command(
#   #DEPENDS always_rebuild # file that never exists
#   OUTPUT ${C_EXE} ${PY_EXE}
#   WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
#   COMMAND dune build --build-dir=${BUILD_DIR})
# add_custom_target(dune-build-generators DEPENDS ${C_EXE} ${PY_EXE})

add_custom_command(
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h
  DEPENDS ${C_EXE}
  COMMAND ${C_EXE} > ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h)
add_custom_target(generate-c-tracing-structs DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.h)
add_dependencies(generate-c-tracing-structs dune-build-generators)

add_custom_command(
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.py
  DEPENDS ${PY_EXE}
  COMMAND ${PY_EXE} > ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.py)
add_custom_target(generate-py-tracing-structs DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/tracing_structs.py)
add_dependencies(generate-py-tracing-structs dune-build-generators)

# finally, turn this into a library so that it can be used downstream (and configure header include path)

add_library(c-tracing-structs INTERFACE)
target_include_directories(c-tracing-structs INTERFACE ${CMAKE_CURRENT_BINARY_DIR})
add_dependencies(c-tracing-structs generate-c-tracing-structs)
