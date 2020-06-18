#include "../util/test.h"
#include "../util/hashtable.h"

#include <stdlib.h>
#include <assert.h>

#define U64_LIT_REF(v) &(uint64_t){ v }

static uint64_t
key_hash_u64( void const * k ) { return *(uint64_t*)k; }

TEST( simple, "hashtable" )
{
  typedef uint64_t key_t;
  typedef uint64_t val_t;

  static const hashtable_functions_t fs = {
    .key_hash = key_hash_u64,
    .key_eq   = NULL,
    .key_del  = NULL,
    .val_del  = NULL,
  };

  hashtable_t * tbl = NEW_HASHTABLE_FUNCS( key_t, val_t, fs );
  REQUIRE( tbl );

  hashtable_error_t e = hashtable_insert( tbl, U64_LIT_REF(20), U64_LIT_REF(200) );
  CHECK( e == HASHTABLE_SUCCESS );

  uint64_t const * v = hashtable_at( tbl, U64_LIT_REF(20) );
  REQUIRE( v );
  CHECK( *v == 200 );

  delete_hashtable( tbl );
}
