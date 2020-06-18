#include "hashtable.h"

#include <assert.h>
#include <stdalign.h>
#include <stdlib.h>
#include <string.h>
#include <xxhash.h>

#define MAX_LOAD   (0.8f)
#define INIT_SLOTS (1024ul)
#define ALIGN( addr, align ) ( ( ((size_t)addr) + ( ((size_t)align) - 1 ) ) & - ((size_t)align) )
#define MAX( a, b ) ( (a) < (b) ? (b) : (a) )

// FIXME check align

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

static void
nodes_at( hashtable_t * tbl,
          size_t        idx,
          bool * *      used,
          void * *      key,
          void * *      value )
{
  size_t nf = node_footprint( tbl->key_footprint, tbl->key_align, tbl->value_footprint, tbl->value_align );
  size_t offset = nf*idx;

  char * mem = (char*)tbl->nodes + offset;
  *used  = (bool*)mem;
  mem   += sizeof(bool);

  mem = (char*)ALIGN( mem, tbl->key_align );
  *key = mem;

  mem += tbl->key_footprint;
  mem = (char*)ALIGN( mem, tbl->value_align );
  *value = mem;
}

hashtable_t *
new_hashtable( size_t                key_footprint,
               size_t                key_align,
               size_t                value_footprint,
               size_t                value_align,
               hashtable_functions_t functions )
{
  hashtable_t * ret = NULL;
  int err = posix_memalign( (void*)&ret, alignof( hashtable_t ), sizeof( *ret ) );
  if (err != 0) return NULL;

  size_t nf = INIT_SLOTS*node_footprint( key_footprint, key_align, value_footprint, value_align );
  err = posix_memalign( &ret->nodes, MAX( key_align, 32 ), nf );
  if( err != 0 ) {
    free( ret );
    return NULL;
  }

  memset( ret->nodes, 0, nf );

  // FIXME enforce power of two

  ret->key_footprint   = key_footprint;
  ret->key_align       = key_align;
  ret->value_footprint = value_footprint;
  ret->value_align     = value_align;
  *(ret->functions)    = functions;
  ret->n_entries       = 0;
  ret->n_slots         = INIT_SLOTS;
  return ret;
}

void
delete_hashtable( hashtable_t * tbl )
{
  if( !tbl ) return;
  free( tbl->nodes );
  free( tbl );
}

hashtable_error_t
hashtable_insert_real( hashtable_t * tbl,
                       void const *  key,
                       void const *  value )
{
  size_t   key_footprint   = tbl->key_footprint;
  size_t   value_footprint = tbl->value_footprint;
  uint64_t entries         = tbl->n_entries;
  uint64_t n_slots         = tbl->n_slots;
  uint64_t mask            = n_slots-1;
  uint64_t hash            = key_hash( tbl, key );
  uint64_t bucket          = hash & mask;

  if( entries >= (size_t)((float)n_slots*MAX_LOAD) ) { // FIXME numerics
    abort();
  }

  while( 1 ) {
    bool * used;
    void * node_key;
    void * node_value;
    nodes_at( tbl, bucket, &used, &node_key, &node_value );

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

  while( 1 ) {
    bool * used;
    void * node_key;
    void * node_value;
    nodes_at( tbl, bucket, &used, &node_key, &node_value );

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
    bool * used;
    void * node_key;
    void * node_value;
    nodes_at( tbl, bucket, &used, &node_key, &node_value );

    // if we hit an empty, there's nothing else to move
    if( !*used ) return;

    uint64_t node_bucket = key_hash( tbl, node_key ) & mask;

    if( node_bucket != bucket ) {
      // the last bucket must be free, else we'd have bailed out
      bool * last_used;
      void * last_key;
      void * last_value;
      nodes_at( tbl, last_bucket, &last_used, &last_key, &last_value );

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

  while( 1 ) {
    bool * used;
    void * node_key;
    void * node_value;
    nodes_at( (void*)tbl, bucket, &used, &node_key, &node_value );
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
                     void * *           out_value )
{
  while( 1 ) {
    if( iter->idx > iter->tbl->n_slots ) return HASHTABLE_FINISH;

    bool * used;
    nodes_at( iter->tbl, iter->idx, &used, (void**)out_key, out_value );

    iter->idx += 1;
    if( *used ) return HASHTABLE_SUCCESS;
  }
}

// FIXME custom allocator
