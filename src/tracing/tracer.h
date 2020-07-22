#pragma once

#include <stdlib.h>

typedef struct tracer tracer_t;

tracer_t *
new_tracer( char const * filename );

void
delete_tracer( tracer_t * tracer );

void
tracer_write( tracer_t * tracer, int id, void const * message, size_t sz );
