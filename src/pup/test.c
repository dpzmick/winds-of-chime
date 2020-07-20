#include "foo.h"
#include "tracer.h"

#include <stdio.h>
#include <stdbool.h>

#define ARRAY_SIZE( arr ) ( sizeof( arr )/sizeof( *arr ) )

int main() {
  int8_t arr[]  = { 1, 2, 3 };
  int8_t buf[10] = {0};

  foo_t foo[1];
  foo_reset( foo, 10, ARRAY_SIZE( arr ), arr, buf );

  printf( "arr_sz = %d\n", foo_get_arr_sz( foo ) );

  tracer_t * tracer = tracer_new();

  frame_timer_t timer[1];
  while( 1 ) {
    uint64_t start = 0;
    uint64_t end   = 0;

    frame_timer_reset( timer, start, end );
    tracer_write_pup( tracer, timer );
  }
}

// parts;
// - automatic (fast) serialization code (done, this is Pup)
// - message bus + message tagging
