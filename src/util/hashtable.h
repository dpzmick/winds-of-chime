#pragma once

// awkward and okayish performance hashtable (probably)

#include <stdalign.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct hashtable      hashtable_t;
typedef struct hashtable_iter hashtable_iter_t;

typedef uint64_t
(*hashtable_key_hash)( void const * k );

typedef bool
(*hashtable_key_eq)( void const * k1,
                     void const * k2);

typedef void
(*hashtable_obj_del)( void * k );

enum {
  HASHTABLE_INIT_SLOTS_DEFAULT = 1024,
};

// no move supported at the moment
// don't store anything self-referential

typedef struct {
  hashtable_key_hash key_hash;
  hashtable_key_eq   key_eq;
  hashtable_obj_del  key_del;
  hashtable_obj_del  val_del;
} hashtable_functions_t;

typedef enum {
  HASHTABLE_SUCCESS,
  HASHTABLE_FINISH,
  HASHTABLE_ERR_ALLOC,
  HASHTABLE_ERR_ALREADY_PRESENT,
} hashtable_error_t;

static const hashtable_functions_t hashtable_functions_empty = {
  .key_hash = NULL,
  .key_eq   = NULL,
  .key_del  = NULL,
  .val_del  = NULL,
};

hashtable_t *
new_hashtable( size_t                key_footprint,
               size_t                key_align,
               size_t                value_footprint,
               size_t                value_align,
               size_t                init_slots,
               hashtable_functions_t functions );

#define NEW_HASHTABLE( K, V )                          \
  new_hashtable( sizeof( K ), alignof( V ),            \
                 sizeof( V ), alignof( V ),            \
                 HASHTABLE_INIT_SLOTS_DEFAULT,         \
                 hashtable_functions_empty )

#define NEW_HASHTABLE_FUNCS( K, V, functions )         \
  new_hashtable( sizeof( K ), alignof( V ),            \
                 sizeof( V ), alignof( V ),            \
                 HASHTABLE_INIT_SLOTS_DEFAULT,         \
                 functions )                           \

#define NEW_HASHTABLE_SLOTS( K, V, slots )             \
  new_hashtable( sizeof( K ), alignof( V ),            \
                 sizeof( V ), alignof( V ),            \
                 slots,                                \
                 hashtable_functions_empty )           \

// calls del on all stored keys and values
void
delete_hashtable( hashtable_t * table );

// stores the value pointer
// memcpy's the key and value using footprints from ctor
// can be treated like a `move`
hashtable_error_t
hashtable_insert_real( hashtable_t * table,
                       void const *  key,
                       void const *  value );

#define hashtable_insert( t, k, v ) \
  hashtable_insert_real( (t), (void const*)(k), (void const*)(v) )

// calls del on stored keys and values
void
hashtable_remove( hashtable_t * table,
                  void const *  key );

// returns NULL if not in table
void const *
hashtable_at( hashtable_t const * table,
              void const *        key );

// returns NULL if not in table
void *
hashtable_at_mut( hashtable_t * table,
                  void const *  key );

size_t
hashtable_size( hashtable_t const * table );

hashtable_iter_t *
hashtable_iterate( hashtable_t * table );

void
hashtable_iter_delete( hashtable_iter_t * iter );

hashtable_error_t
hashtable_iter_next( hashtable_iter_t * iter,
                     void const * *     out_key,
                     void const * *     out_value );
