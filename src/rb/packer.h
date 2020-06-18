#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/* Pack values into something suitable for placement into a rb_log (using the specified schema).
   The rb_log values are not compressed or manipulated in any way.

   The intention is that a sufficiently-optimizing compiler can convert each
   insertion to the rb_log into a very small number of instructions.

   Decoding the packed data requires access to the schema. */

/* forward decl */
typedef struct rb_schema rb_schema_t;

typedef struct rb_packer {
  rb_schema_t const * schema;
  uint32_t            buffer[];
} rb_packer_t;

typedef struct rb_packer_transaction {
  rb_packer_t * packer;
  uint64_t      offset;
} rb_packer_transaction_t;

rb_packer_t *
new_rb_packer( rb_schema_t const * schema )
{
  rb_packer_t * ret = calloc( 1, sizeof( *ret )+4096 );
  return ret;
}

void
delete_rb_packer( rb_packer_t * packer )
{
  free( packer );
}

rb_packer_transaction_t
rb_packer_start( rb_packer_t * packer )
{
  return (rb_packer_transaction_t){.packer = packer, .offset = 0};
}

void const *
rb_packer_finalize( rb_packer_transaction_t t )
{
  return t.packer->buffer;
}

static inline bool
rb_packer_start_object( rb_packer_transaction_t * t,
                        uint64_t                  object_id )
{
  memcpy( t->packer->buffer + t->offset, &object_id, sizeof( object_id ) );
  t->offset += sizeof( object_id );
  return true;
}

static inline bool
rb_packer_add_u32( rb_packer_transaction_t * t,
                   uint32_t                  value )
{
  memcpy( t->packer->buffer + t->offset, &value, sizeof( value ) );
  t->offset += sizeof( value );
  return true;
}

/* bool */
/* rb_packer_add_u64( rb_packer_t * packer, */
/*                    uint64_t      value ) */
/* { */
/*   memcpy( packer->buffer + packer->offset, &value, sizeof( value ) ); */
/*   packer->offset += sizeof( value ); */
/*   return true; */
/* } */
