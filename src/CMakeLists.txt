add_subdirectory(shaders)

add_subdirectory(util)
add_subdirectory(pup)
#add_subdirectory(test)

add_executable(main common.c app.c main.c)
target_link_libraries(main util volk glfw Threads::Threads)
