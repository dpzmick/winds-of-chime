#pragma once

#include <stdbool.h>

typedef void (*test_function_t)( );

void
test_register( char const * name, char const * tags, test_function_t fp );

void __attribute__((noinline))
test_check_result( bool cond, char const * file, int line, char const * info );

void __attribute__((noinline))
test_require_result( bool cond, char const * file, int line, char const * info );

void
run_tests( void );

#define CHECK( cond )   test_check_result  ( !!(cond), __FILE__, __LINE__, "" #cond );
#define REQUIRE( cond ) test_require_result( !!(cond), __FILE__, __LINE__, "" #cond );

#define TEST_PASTE2( x, y, z ) x ## y ## z
#define TEST_PASTE( x, y, z ) TEST_PASTE2( x, y, z )

#define TEST( _macro_test_name, _macro_test_tags )                      \
  static void TEST_PASTE(test_, _macro_test_name, __LINE__)( void );    \
                                                                        \
  static void __attribute__((constructor))                              \
  TEST_PASTE(register_test_, _macro_test_name, __LINE__)( void ) {      \
    test_register( "" #_macro_test_name,                                \
                   _macro_test_tags,                                    \
                   TEST_PASTE(test_, _macro_test_name, __LINE__) );     \
  }                                                                     \
                                                                        \
  /* actually define the function */                                    \
  static void TEST_PASTE(test_, _macro_test_name, __LINE__)( void )     \

// asd
