#include "foo.h"

#include <stdio.h>

#define ARRAY_SIZE( arr ) ( sizeof( arr )/sizeof( *arr ) )

typedef struct {
  bool idk;
} tracer_t;

tracer_t *
tracer_new( void );

void
tracer_write( tracer_t * tracer, int id, void const * message, size_t sz );

int main() {
  int8_t arr[]  = { 1, 2, 3 };
  int8_t buf[10] = {0};

  foo_t foo[1];
  foo_init( foo, 10, ARRAY_SIZE( arr ), arr, buf );

  printf( "arr_sz = %d\n", foo_get_arr_sz( foo ) );

  tracer_t * tracer = tracer_new();

  frame_timer_t timer[1];
  while( 1 ) {
    uint64_t start = 0;
    uint64_t end   = 0;

    frame_timer_init( timer, start, end );
    tracer_write_pup( tracer, timer );
  }
}

// parts;
// - automatic (fast) serialization code (done, this is Pup)
// - message bus + message tagging
