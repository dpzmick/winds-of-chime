add_executable(unit_test
  unit_test.c
  test_hashtable.c)

target_link_libraries(unit_test test util)
