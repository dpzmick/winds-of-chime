#include "test.h"

#include "../common.h"

#include <pthread.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <xxhash.h>

typedef struct test test_t;

static test_t *  all_program_tests = NULL;
static pthread_t thread[1];

struct test {
  test_t *        next;
  char const *    name;
  char const *    tags;
  test_function_t fp;
};

static void __attribute__((destructor))
test_cleanup( void )
{
  test_t * next = all_program_tests;
  while( 1 ) {
    if( !next ) break;
    test_t * last = next;
    next = last->next;
    free( last );
  }
}

void
test_register( char const *    name,
               char const *    tags,
               test_function_t fp )
{
  test_t * new_test = malloc( sizeof( *new_test ) );
  if( !new_test ) abort(); // FIXME logging

  new_test->next = all_program_tests;
  new_test->name = name;
  new_test->tags = tags;
  new_test->fp   = fp;

  all_program_tests = new_test;
}

void
test_check_result( bool         cond,
                   char const * file,
                   int          line,
                   char const * info )
{
  if( LIKELY( cond ) ) return;

  printf( "%s:%d CHECK failed: (%s)\n", file, line, info );
  __builtin_trap();
}

void
test_require_result( bool         cond,
                     char const * file,
                     int          line,
                     char const * info )
{
  if( LIKELY( cond ) ) return;

  printf( "%s:%d REQUIRE failed: (%s)\n", file, line, info );
  __builtin_trap();
  pthread_exit( NULL );
}

void *
thread_func( void * fp_void )
{
  test_function_t fp = (test_function_t)fp_void;
  fp();
  return NULL;
}

void run_tests( void )
{
  test_t * t = all_program_tests;
  while( 1 ) {
    if( !t ) break;
    printf("test: %s\n", t->name);

    int err = pthread_create( thread, NULL, thread_func, t->fp );
    if( err != 0 ) abort();

    err = pthread_join( *thread, NULL ); // FIXME reuse the thread
    if( err != 0 ) abort();

    t = t->next;
  }
}
