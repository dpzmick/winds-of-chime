#include "hashtable.h"

#include <assert.h>
#include <stdalign.h>
#include <stdlib.h>
#include <string.h>
#include <xxhash.h>
#include <stdio.h>

#define MAX_LOAD   (0.8f)
#define ALIGN( addr, align ) ( ( ((size_t)addr) + ( ((size_t)align) - 1 ) ) & - ((size_t)align) )
#define MAX( a, b ) ( (a) < (b) ? (b) : (a) )

// FIXME check align
// FIXME likely/unlikely

struct hashtable {
  size_t                key_footprint;
  size_t                key_align;
  size_t                value_footprint;
  size_t                value_align;
  hashtable_functions_t functions[1];
  size_t                n_entries;
  size_t                n_slots;
  void *                nodes;
};

struct hashtable_iter {
  hashtable_t * tbl;
  size_t        idx;
};

static uint64_t
key_hash( hashtable_t const * tbl,
          void const *        key )
{
  hashtable_key_hash hash_f = tbl->functions->key_hash;
  uint64_t           f      = tbl->key_footprint;

  if( hash_f ) return hash_f( key );
  return XXH64( key, f, 0xcafebabe );
}

static bool
key_eq( hashtable_t const * tbl,
        void const *        a,
        void const *        b )
{
  hashtable_key_eq key_eq = tbl->functions->key_eq;
  uint64_t         f      = tbl->key_footprint;

  if( key_eq ) return key_eq( a, b );
  return 0 == memcmp( a, b, f );
}

static void
key_del( hashtable_t const * tbl,
         void *              key )
{
  hashtable_obj_del del = tbl->functions->key_del;
  if( del ) del( key );
}

static void
val_del( hashtable_t const * tbl,
         void *              val )
{
  hashtable_obj_del del = tbl->functions->val_del;
  if( del ) del( val );
}

// each node contains a bool, then a densely packed key/value pair

static size_t
node_footprint( size_t key_footprint, size_t key_align,
                size_t value_footprint, size_t value_align )
{
  size_t footprint = sizeof( bool );
  footprint  = ALIGN( footprint, key_align );
  footprint += key_footprint;
  footprint  = ALIGN( footprint, value_align );
  footprint += value_footprint;
  footprint  = ALIGN( footprint, key_align );

  // footprint | footprint
  //             ^ needs to be aligned for a key
  return footprint;
}

// grabbing these arrays ahead of a loop, then indexing into them carefully
// results in much better codegen
static void
get_arrays( hashtable_t const * tbl,
            char * *            used_array,
            char * *            key_array,
            char * *            value_array,
            size_t *            skip )
{
  size_t key_footprint   = tbl->key_footprint;
  size_t key_align       = tbl->key_align;
  size_t value_footprint = tbl->value_footprint;
  size_t value_align     = tbl->value_align;
  char * nodes           = tbl->nodes;

  *used_array  = nodes;
  *key_array   = (char*)ALIGN( nodes+sizeof(bool), key_align );
  *value_array = (char*)ALIGN( *key_array + key_footprint, value_align );
  *skip = node_footprint( key_footprint, key_align, value_footprint, value_align );
}

// consider allowing user to put hashtable in their own memory

hashtable_t *
new_hashtable( size_t                key_footprint,
               size_t                key_align,
               size_t                value_footprint,
               size_t                value_align,
               size_t                init_slots,
               hashtable_functions_t functions ) // FIXME error code
{
  hashtable_t * ret = NULL;
  int err = posix_memalign( (void*)&ret, alignof( hashtable_t ), sizeof( *ret ) );
  if (err != 0) return NULL;

  size_t nf = init_slots*node_footprint( key_footprint, key_align, value_footprint, value_align );
  err = posix_memalign( &ret->nodes, MAX( key_align, 32 ), nf );
  if( err != 0 ) {
    free( ret );
    return NULL;
  }

  // FIXME check that init slots is power of two

  memset( ret->nodes, 0, nf );

  ret->key_footprint   = key_footprint;
  ret->key_align       = key_align;
  ret->value_footprint = value_footprint;
  ret->value_align     = value_align;
  *(ret->functions)    = functions;
  ret->n_entries       = 0;
  ret->n_slots         = init_slots;
  return ret;
}

void
delete_hashtable( hashtable_t * tbl )
{
  if( !tbl ) return;
  // FIXME delete keys and values
  free( tbl->nodes );
  free( tbl );
}

static hashtable_error_t
insert_inner( hashtable_t * tbl,
              void const *  key,
              void const *  value,
              uint64_t      n_slots,
              char *        used_array,
              char *        key_array,
              char *        value_array,
              size_t        skip )
{
  uint64_t mask            = n_slots-1;
  uint64_t hash            = key_hash( tbl, key );
  uint64_t bucket          = hash & mask;
  size_t   key_footprint   = tbl->key_footprint;
  size_t   value_footprint = tbl->value_footprint;

  while( 1 ) {
    bool * used       = (bool*)(used_array + bucket*skip);
    void * node_key   = key_array + bucket*skip;
    void * node_value = value_array + bucket*skip;

    if( !*used ) {
      *used = true;
      memcpy( node_key,   key,   key_footprint );
      memcpy( node_value, value, value_footprint );
      tbl->n_entries += 1;
      return HASHTABLE_SUCCESS;
    }

    if( key_eq( tbl, key, node_key ) ) {
      return HASHTABLE_ERR_ALREADY_PRESENT;
    }

    bucket += 1;
    bucket = bucket & mask;
  }

  __builtin_unreachable();
}

static hashtable_error_t
resize_table( hashtable_t * tbl )
{
  hashtable_error_t err = HASHTABLE_SUCCESS;

  // allocate new entires, rehash
  // we don't update the table until the very end in case of error, to allow for
  // user to cleanup
  size_t key_footprint   = tbl->key_footprint;
  size_t key_align       = tbl->key_align;
  size_t value_footprint = tbl->value_footprint;
  size_t value_align     = tbl->value_align;
  size_t n_slots         = tbl->n_slots;

  // new values
  size_t new_slots = n_slots * 2;
  size_t nf        = new_slots*node_footprint( key_footprint, key_align, value_footprint, value_align );

  char * nodes = NULL;
  int    ret   = posix_memalign( (void**)&nodes, MAX( key_align, 32 ), nf );
  if( ret != 0 ) {
    err = HASHTABLE_ERR_ALLOC;
    goto fail;
  }

  size_t skip;
  char * old_used_array;
  char * old_key_array;
  char * old_value_array;
  get_arrays( tbl, &old_used_array, &old_key_array, &old_value_array, &skip );

  // figure out new offsets
  char * new_used_array  = nodes + (old_used_array - (char*)tbl->nodes);
  char * new_key_array   = nodes + (old_key_array - (char*)tbl->nodes);
  char * new_value_array = nodes + (old_value_array - (char*)tbl->nodes);

  for( size_t i = 0; i < n_slots; ++i ) {
    bool         old_used  = *(old_used_array + i*skip);
    void const * old_key   = old_key_array + i*skip;
    void const * old_value = old_value_array + i*skip;

    // FIXME support move semantics here, objects need to be moved exactly once
    // into final resting place, and we need to call a used-provided move-ctor
    // as part of the move

    if( old_used ) {
      err = insert_inner( tbl, old_key, old_value, n_slots, new_used_array, new_key_array, new_value_array, skip );
      if( err != HASHTABLE_SUCCESS ) goto fail;
    }
  }

  free( tbl->nodes );
  tbl->n_slots = new_slots;
  tbl->nodes   = nodes;

  return HASHTABLE_SUCCESS;

fail:
  if( nodes ) free( nodes );
  return err;
}

hashtable_error_t
hashtable_insert_real( hashtable_t * tbl,
                       void const *  key,
                       void const *  value )
{
  uint64_t entries = tbl->n_entries;
  uint64_t n_slots = tbl->n_slots;

  if( entries >= (size_t)((float)n_slots*MAX_LOAD) ) { // FIXME numerics
    hashtable_error_t e = resize_table( tbl );
    if( e != HASHTABLE_SUCCESS ) return e;
  }

  char * used_array;
  char * key_array;
  char * value_array;
  size_t skip;
  get_arrays( tbl, &used_array, &key_array, &value_array, &skip );
  return insert_inner( tbl, key, value, n_slots, used_array, key_array, value_array, skip );
}

void
hashtable_remove( hashtable_t * tbl,
                  void const *  key )
{
  // make sure that the key is actually in here

  uint64_t n_slots      = tbl->n_slots;
  uint64_t mask         = n_slots-1;
  uint64_t hash         = key_hash( tbl, key );
  uint64_t bucket       = hash & mask;
  uint64_t last_bucket  = (uint64_t)-1;
  uint64_t start_bucket = bucket;

  char * used_array;
  char * key_array;
  char * value_array;
  size_t skip;
  get_arrays( tbl, &used_array, &key_array, &value_array, &skip );

  while( 1 ) {
    bool * used       = (bool*)(used_array + bucket*skip);
    void * node_key   = key_array + bucket*skip;
    void * node_value = value_array + bucket*skip;

    // bail if we hit an empty before finding the key
    if( !*used ) return;

    // if this is the key, remove it from the hash table, then cleanup.
    if( key_eq( tbl, node_key, key ) ) {
      *used = false;
      key_del( tbl, node_key );
      val_del( tbl, node_value );
      break;
    }

    last_bucket  = bucket;
    bucket      += 1;
    bucket       = bucket & mask;

    // shouldn't really happen, but if it does we're done
    if( bucket == start_bucket ) return;
  }

  last_bucket  = bucket;
  bucket      += 1;
  bucket       = bucket & mask;

  while( 1 ) {
    // if the key here isn't supposed to be here, move it back one slot
    bool * used       = (bool*)(used_array + bucket*skip);
    void * node_key   = key_array + bucket*skip;
    void * node_value = value_array + bucket*skip;

    // if we hit an empty, there's nothing else to move
    if( !*used ) return;

    uint64_t node_bucket = key_hash( tbl, node_key ) & mask;

    if( node_bucket != bucket ) {
      // the last bucket must be free, else we'd have bailed out
      bool * last_used  = (bool*)(used_array + last_bucket*skip);
      void * last_key   = key_array + last_bucket*skip;
      void * last_value = value_array + last_bucket*skip;

      *last_used = true;
      memcpy( last_key,   node_key,   tbl->key_footprint );
      memcpy( last_value, node_value, tbl->value_footprint );

      *used = false;
    }

    last_bucket  = bucket;
    bucket      += 1;
    bucket       = bucket & mask;
  }
}

void const *
hashtable_at( hashtable_t const * tbl,
              void const *        key )
{
  uint64_t n_slots = tbl->n_slots;
  uint64_t mask    = n_slots-1;
  uint64_t hash    = key_hash( tbl, key );
  uint64_t bucket  = hash & mask;

  char * used_array;
  char * key_array;
  char * value_array;
  size_t skip;
  get_arrays( tbl, &used_array, &key_array, &value_array, &skip );

  while( 1 ) {
    bool * used       = (bool*)(used_array + bucket*skip);
    void * node_key   = key_array + bucket*skip;
    void * node_value = value_array + bucket*skip;

    if( !*used ) return NULL;
    if( key_eq( tbl, key, node_key ) ) return node_value;

    bucket += 1;
    bucket = bucket & mask;
  }

  __builtin_unreachable();
}

size_t
hashtable_size( hashtable_t const * tbl )
{
  return tbl->n_entries;
}

hashtable_iter_t *
hashtable_iterate( hashtable_t * tbl )
{
  hashtable_iter_t * iter = calloc( 1, sizeof( *iter ) );
  if (!iter) return NULL;

  iter->tbl = tbl;
  iter->idx = 0;

  return iter;
}

void
hashtable_iter_delete( hashtable_iter_t * iter )
{
  free( iter );
}

hashtable_error_t
hashtable_iter_next( hashtable_iter_t * iter,
                     void const * *     out_key,
                     void const * *     out_value )
{
  size_t n_slots = iter->tbl->n_slots;

  char * used_array;
  char * key_array;
  char * value_array;
  size_t skip;
  get_arrays( iter->tbl, &used_array, &key_array, &value_array, &skip );

  while( 1 ) {
    size_t idx = iter->idx;
    if( idx > n_slots ) return HASHTABLE_FINISH;
    iter->idx += 1;

    bool const * used       = (bool*)(used_array + idx*skip);
    void const * node_key   = key_array + idx*skip;
    void const * node_value = value_array + idx*skip;

    // got one
    if( *used ) {
      if (out_key)   *out_key   = node_key;
      if (out_value) *out_value = node_value;
      return HASHTABLE_SUCCESS;
    }
  }
}

// FIXME custom allocator
// FIXME move semantics
