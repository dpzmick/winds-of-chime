#include "tracer.h"

#include <stdio.h>

typedef struct __attribute__((packed)) {
  int    tag;
  size_t size;
} message_hdr_t;

struct tracer {
  FILE * fp;
};

tracer_t *
new_tracer( char const * filename )
{
  FILE* fp = fopen( filename, "w" );
  tracer_t * tracer = malloc( sizeof( tracer_t ) );
  tracer->fp = fp;
  return tracer;
}

void
delete_tracer( tracer_t * tracer )
{
  fclose( tracer->fp );
}

void
tracer_write( tracer_t * tracer, int id, void const * message, size_t sz )
{
  message_hdr_t hdr[1];
  hdr->tag  = id;
  hdr->size = sz;

  //printf("id: %d sz: %zu\n", id, sz);

  fwrite( hdr, sizeof( hdr ), 1, tracer->fp );
  fwrite( message, sz, 1, tracer->fp );
}
