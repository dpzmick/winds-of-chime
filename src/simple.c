#include "rb/pup.h"

#include <stdlib.h>

int main() {
  pup_schema_field_t fields[] = {{
    .type = PUP_FIELD_TYPE_U64,
    .max_elements = 1,
  }, {
    .type = PUP_FIELD_TYPE_U64,
    .max_elements = 10,
  }};

  pup_schema_object_spec_t objects[] = {{
    .n_fields = 1,
    .fields = &fields[0],
  }, {
    .n_fields = 2,
    .fields   = fields,
  }};

  pup_schema_spec_t spec[] = {{
    .n_objects = 2,
    .objects   = objects,
  }};

  void* mem = malloc( pup_schema_footprint( spec ) );
  if( !mem ) abort();

  pup_schema_t * schema = new_pup_schema( mem, spec, NULL );
  if( !schema ) abort();

  // create a buffer to serialize messages into
  void* buffer = malloc( pup_buffer_footprint( schema ) );
}
