add_library(test STATIC test.c)
target_link_libraries(test xxhash)

add_library(util STATIC hashtable.c)
target_link_libraries(util xxhash)
