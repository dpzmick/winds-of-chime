add_executable(main common.c log.c app.c main.c)
target_link_libraries(main volk)
