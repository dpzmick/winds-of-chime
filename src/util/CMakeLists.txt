add_library(test STATIC test.c)
target_link_libraries(test Threads::Threads xxhash)

add_library(util STATIC hashtable.c log.c)
target_link_libraries(util xxhash)
