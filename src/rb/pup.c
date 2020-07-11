#include "pup.h"
#include "../util/hashtable.h"

#include <string.h>

// FIXME put somewhere common
#define MAX( x, y ) ( (x) < (y) ? (y) : (x) )

// layout on disk, also what goes into hashtable if possible
typedef struct {
  size_t             n_fields;
  pup_schema_field_t fields[];
} obj_spec_t;

struct pup_schema {
  size_t n_objects;
  size_t object_size;  // all trailing specs are sz of largest
  // trailing array of obj_spec_t, all of size above
};

static inline size_t
obj_spec_footprint( size_t n_fields )
{
  return sizeof( obj_spec_t ) + n_fields * sizeof( pup_schema_field_t );
}

static inline size_t
schema_object_size( pup_schema_spec_t const * spec )
{
  size_t max_n_field = 0;
  for( size_t i = 0; i < spec->n_objects; ++i ) {
    max_n_field = MAX( spec->objects[i].n_fields, max_n_field );
  }

  return obj_spec_footprint( max_n_field );
}

static obj_spec_t *
schema_get_objs( pup_schema_t * schema )
{
  return (obj_spec_t*)((char*)schema + sizeof( schema ));
}

static obj_spec_t *
schema_get_obj_spec( pup_schema_t * schema,
                     size_t         idx )
{
  return schema_get_objs( schema ) + idx * schema->object_size;
}

size_t
pup_schema_footprint( pup_schema_spec_t const * spec )
{
  if( !spec ) return 0;

  size_t obj_footprint = schema_object_size( spec );
  return sizeof( pup_schema_t ) + spec->n_objects * obj_footprint;
}

size_t
pup_schema_align( void )
{
  return alignof( pup_schema_t );
}

pup_schema_t *
new_pup_schema( void *                    mem,
                pup_schema_spec_t const * spec,
                pup_error_t *             opt_err )
{
  pup_error_t err = PUP_OK;

  if( !mem )  { err = PUP_ERR_INVALID; goto fail; }
  if( !spec ) { err = PUP_ERR_INVALID; goto fail; }

  size_t obj_footprint = schema_object_size( spec );

  pup_schema_t * schema = mem;
  schema->n_objects   = spec->n_objects;
  schema->object_size = obj_footprint;

  for( size_t i = 0; i < spec->n_objects; ++i ) {
    pup_schema_object_spec_t const * ospec = &spec->objects[i];
    obj_spec_t *                     obj   = schema_get_obj_spec( schema, i );

    obj->n_fields = ospec->n_fields;
    memcpy( obj->fields, ospec->fields, ospec->n_fields * sizeof( pup_schema_field_t ) );
  }

  return schema;
fail:
  if( opt_err ) *opt_err = err;
  return NULL;
}
