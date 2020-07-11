#pragma once

// pup: pointless ugly packer

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* our types */
typedef struct rb_schema_field rb_schema_field_t;
typedef struct rb_schema       rb_schema_t;

rb_schema_field_t *
new_rb_schema_field_u64( char const * field_name );

rb_schema_field_t *
new_rb_schema_field_u32( char const * field_name );

rb_schema_field_t *
new_rb_schema_field_object( char const * field_name,
                            char const * object_type );

void
delete_rb_schema_field( rb_schema_field_t * field );

// ---

rb_schema_t *
new_rb_schema( void );

void
delete_rb_schema( rb_schema_t * schema );

bool
rb_schema_add_object( rb_schema_t *       schema,
                      uint64_t            object_id,
                      char const *        object_name,
                      rb_schema_field_t * fields,
                      size_t              n_fields );
