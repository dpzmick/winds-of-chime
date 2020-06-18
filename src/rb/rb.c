#include "schema.h"

#include "../util/hashtable.h"

#include <stdlib.h>
#include <string.h>


static hashtable_functions_t schema_htbl_functions = {
};

typedef enum {
  RB_FIELD_TYPE_U32,
  RB_FIELD_TYPE_U64,
  RB_FIELD_TYPE_OBJECT,
} rb_field_type_t;

struct rb_schema_field {
  rb_field_type_t ft;
  char const *    field_name;
  char const *    opt_object_name;
};

typedef struct rb_schema_object {
  char *              object_name;
  rb_schema_field_t * fields;
  size_t              n_fields;
} rb_schema_object_t;

static uint64_t rb_schema_object_hash( rb_schema_object_t const * obj )
{
  // hash the object name
}

struct rb_schema {
  hashtable_t * objects; // object_id -> schema_object
};

static rb_schema_field_t *
new_rb_schema_field_common( char const *    field_name,
                            rb_field_type_t ft )
{
  rb_schema_field_t * ret = malloc( sizeof( *ret ) );
  if( !ret ) return NULL;

  ret->ft = ft;
  ret->field_name = strdup( field_name );
  if( !ret->field_name ) {
    free( ret );
    return NULL;
  }

  // FIXME establish a max lenght for a field_name

  return ret;
}

rb_schema_field_t *
new_rb_schema_field_u32( char const * field_name )
{
  return new_rb_schema_field_common( field_name, RB_FIELD_TYPE_U32 );
}

rb_schema_field_t *
new_rb_schema_field_u64( char const * field_name )
{
  return new_rb_schema_field_common( field_name, RB_FIELD_TYPE_U64 );
}

rb_schema_field_t *
new_rb_schema_field_object( char const * field_name,
                            char const * object_type )
{
  rb_schema_field_t * ret = new_rb_schema_field_common( field_name, RB_FIELD_TYPE_OBJECT );
  if( !ret ) return NULL;

  ret->opt_object_name = strdup( object_type );
  if( !ret->opt_object_name ) {
    free( ret );
    return NULL;
  }

  // FIXME establish a max lenght for an object_type

  return ret;
}

rb_schema_t *
new_rb_schema( void )
{
  return NULL;
}

void
delete_rb_schema( rb_schema_t * schema )
{
  if( !schema ) return;
  free( schema );
}

bool
rb_schema_add_object( rb_schema_t *       schema,
                      uint64_t            object_id,
                      char const *        object_name,
                      rb_schema_field_t * fields,
                      size_t              n_fields )
{
 
}
