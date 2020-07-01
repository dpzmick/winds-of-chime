#include "../util/test.h"
#include "../util/hashtable.h"

#include <stdlib.h>
#include <assert.h>

#define U64_LIT_REF(v) &(uint64_t){ v }
#define STR_BUF_LIT(lit, sz) (char[sz]){ lit } // not sure the entire buffer defined to get zeroed

static_assert( sizeof( STR_BUF_LIT( "asd", 32 ) ) == 32, "cool trick" );

TEST( single_value, "hashtable" )
{
  typedef uint64_t key_t;
  typedef uint64_t val_t;

  hashtable_t * tbl = NEW_HASHTABLE( key_t, val_t );
  REQUIRE( tbl );

  hashtable_error_t e = hashtable_insert( tbl, U64_LIT_REF(20), U64_LIT_REF(200) );
  CHECK( e == HASHTABLE_SUCCESS );

  uint64_t const * v = hashtable_at( tbl, U64_LIT_REF(20) );
  REQUIRE( v );
  CHECK( *v == 200 );

  delete_hashtable( tbl );
}

TEST( remove, "hashtable" )
{
  typedef uint64_t key_t;
  typedef uint64_t val_t;

  hashtable_t * tbl = NEW_HASHTABLE( key_t, val_t );
  REQUIRE( tbl );

  hashtable_error_t e = hashtable_insert( tbl, U64_LIT_REF(20), U64_LIT_REF(200) );
  CHECK( e == HASHTABLE_SUCCESS );

  uint64_t const * v = hashtable_at( tbl, U64_LIT_REF(20) );
  REQUIRE( v );
  CHECK( *v == 200 );

  e = hashtable_insert( tbl, U64_LIT_REF(20), U64_LIT_REF(100) );
  CHECK( e == HASHTABLE_ERR_ALREADY_PRESENT );

  hashtable_remove( tbl, U64_LIT_REF(20) );

  v = hashtable_at( tbl, U64_LIT_REF(20) );
  CHECK( !v );

  e = hashtable_insert( tbl, U64_LIT_REF(20), U64_LIT_REF(100) );
  CHECK( e == HASHTABLE_SUCCESS );

  v = hashtable_at( tbl, U64_LIT_REF(20) );
  REQUIRE( v );
  CHECK( *v == 100 );

  delete_hashtable( tbl );
}

TEST( string_key, "hashtable" )
{
#define LN 32
#define S(l) STR_BUF_LIT(l,LN)

  hashtable_t * tbl = NEW_HASHTABLE( char[LN], uint64_t );
  REQUIRE( tbl );

  hashtable_error_t e = hashtable_insert( tbl, S("asd"), U64_LIT_REF(200) );
  CHECK( e == HASHTABLE_SUCCESS );

  uint64_t const * v = hashtable_at( tbl, S("asd") );
  REQUIRE( v );
  CHECK( *v == 200 );

  delete_hashtable( tbl );

#undef S
#undef LN
}

TEST( insert_many, "hashtable" )
{
  hashtable_t * tbl = NEW_HASHTABLE_SLOTS( uint64_t, uint64_t, 1 );
  REQUIRE( tbl );

  for( size_t i = 0; i < 100; ++i ) {
    hashtable_error_t e = hashtable_insert( tbl, &i, U64_LIT_REF(200-i) );
    REQUIRE( e == HASHTABLE_SUCCESS );
  }

  bool gotem[100];
  hashtable_iter_t * iter = hashtable_iterate( tbl );
  while( 1 ) {
    uint64_t const * key;
    uint64_t const * value;

    hashtable_error_t e = hashtable_iter_next( iter, (void const**)&key, (void const**)&value );
    if( e == HASHTABLE_FINISH ) break;
    REQUIRE( e == HASHTABLE_SUCCESS );

    REQUIRE( *key < 100 );
    REQUIRE( *value == 200-*key );
    gotem[*key] = true;
  }
}
