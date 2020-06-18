#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* shm region containing a sliding buffer of events */

typedef struct rb_log_writer rb_log_writer_t;
typedef struct rb_log_reader rb_log_reader_t;
typedef struct rb_schema     rb_schema_t;
typedef struct rb_packer     rb_packer_t;

typedef enum {
  RB_FIELD_TYPE_U32,
  RB_FIELD_TYPE_U64,
  RB_FIELD_TYPE_OBJECT,
} rb_field_type_t;

typedef struct rb_schema_field_type rb_schema_field_type_t;
struct rb_schema_field_type {
  rb_field_type_t type;
  char const *    opt_object_field_name;
};

rb_schema_t *
new_rb_schema( void );

void
delete_rb_schema( rb_schema_t * schema );

bool
rb_schema_add_object( rb_schema_t *                  schema,
                      uint64_t                       object_id,
                      char const *                   object_name,
                      uint32_t                       object_field_count,
                      char const * const *           field_names,
                      rb_schema_field_type_t const * field_types );
// -----

rb_log_writer_t *
new_rb_log_writer( char const *  path,
                   rb_schema_t * schema,
                   uint32_t      n_slots );

void
delete_rb_log_writer( rb_log_writer_t * log );

void
rb_insert( rb_log_writer_t * wtr,
           uint64_t          object_id,
           void *            packed_object );
