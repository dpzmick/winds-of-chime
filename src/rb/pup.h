#pragma once

/* pup: pointless ugly packer

   Pack simple values into something simple. Designed such that a sufficiently
   optimizing compiler can pack/unpack values very efficiently.

   Decoding and encoding the packed data requires access to the schema.

   Populating the objects must be done accoding to the schema. Validation is
   only performed in debugging builds of this library */

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* our types */
typedef struct pup_schema pup_schema_t;
typedef struct pup_packer pup_packer_t;

typedef enum {
  PUP_OK,
  PUP_ERR_ALLOC,
  PUP_ERR_SCHEMA_DUPLICATE_OBJECT,
  PUP_ERR_INVALID,
} pup_error_t;

// ---
// A schema is a collection of objects which might be present in some data to
// encode/decode.
//
// Every object contained in the schema is given a type name and an ID.
//
// IDs are user assigned (to avoid lookup tables in fast path) and must be
// unique

typedef enum {
  PUP_FIELD_TYPE_U32,
  PUP_FIELD_TYPE_U64,
} pup_field_type_t;

typedef struct {
  pup_field_type_t type;
  size_t           max_elements; // 1 for single value, > 1 for array
} pup_schema_field_t;

typedef struct {
  size_t               n_fields;
  pup_schema_field_t * fields;
} pup_schema_object_spec_t;

typedef struct {
  size_t                     n_objects;
  pup_schema_object_spec_t * objects;
} pup_schema_spec_t;

size_t
pup_schema_footprint( pup_schema_spec_t const * spec );

size_t
pup_schema_align( void );

pup_schema_t *
new_pup_schema( void *                    mem,
                pup_schema_spec_t const * spec,
                pup_error_t *             opt_err );

char const *
pup_schema_serialize( pup_schema_t const * spec,
                      size_t *             out_buffer_size );

// ---
// A packer is used to produce buffers containing packed values.
// Creating a packer allocates a buffer large enough for the largest possibe
// structure in the schema.

#ifndef PUP_DEBUG
struct pup_packer {
  char * next;
};
#else
struct pup_packer {
  char *         next;
  pup_schema_t * schema;
  size_t         object_idx;
  size_t         next_field_offset;
};
#endif

size_t
pup_buffer_footprint( pup_schema_t * schema );

pup_packer_t
pup_packer_start( pup_schema_t * schema,
                  char *         buffer );

// return the actual size used
size_t
pup_packer_finalize( pup_packer_t packer );

// all methods will assert ifdef PUP_DEBUG if the insertion order is incorrect
// or other errors are detected. ifndef PUP_DEBUG, no validation is performed
pup_packer_t
pup_packer_start_object( pup_packer_t packer,
                         uint64_t     schema_idx );

void
pup_packer_finish_object( pup_packer_t * packer );

void
pup_packer_add_u32( pup_packer_t * packer,
                    uint32_t       value );

void
pup_packer_add_u64( pup_packer_t * packer,
                    uint64_t       value );


// FIXME arrays are awkward
