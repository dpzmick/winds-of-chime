#include "rb_log.h"

#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define MAGIC_MAKER(c1, c2, c3, c4, c5, c6, c7, c8)                   \
  (uint64_t)c1                                                        \
  | (uint64_t)c2 << 8                                                 \
  | (uint64_t)c3 << 16                                                \
  | (uint64_t)c4 << 24                                                \
  | (uint64_t)c5 << 32                                                \
  | (uint64_t)c6 << 40                                                \
  | (uint64_t)c7 << 48                                                \
  | (uint64_t)c8 << 56


#define RB_LOG_MAGIC MAGIC_MAKER( 'd', 'z', 'm', 'i', 'c', 'k', 'r', 'b')
#define UNUSED_ENTRY UINT32_MAX

typedef struct __attribute__((packed))
{
  uint64_t magic;

  uint32_t head;
  uint32_t tail;

  uint32_t n_slots;
  uint32_t entries_offset;

  /* \0 terminated field-name strings, as many as the user wants */
  /* possible alignment */
  /* n_slots worth of (uint32_t, uint32_t) entries */
} rb_log_t;

typedef struct __attribute__((packed))
{
  uint32_t field;
  uint32_t value;
} log_entry_t;

static char *
log_fname_buffer( rb_log_t * log )
{
  return (char*)(log + 1);
}

static log_entry_t *
log_entries( rb_log_t * log )
{
  return (log_entry_t*)((char*)(log + 1) + log->entries_offset);
}

struct rb_log_writer
{
  uint64_t   footprint;
  rb_log_t * log;
};

rb_log_writer_t *
new_rb_log_writer( char const *         path,
                   char const * const * field_names,
                   uint32_t             field_names_len,
                   uint32_t             n_slots )
{
  int               ret = 0;
  int               fd  = -1;
  rb_log_writer_t * wtr = NULL;

  if( field_names_len > UINT32_MAX-1 ) goto fail;

  uint64_t strings_region_sz = 0;
  for( uint32_t i = 0; i < field_names_len; ++i ) {
    strings_region_sz += strlen( field_names[i] ) + 1;
  }

  // won't fit in size field
  if( strings_region_sz > UINT32_MAX ) goto fail;

  uint64_t footprint = sizeof( rb_log_t );
  footprint += strings_region_sz;
  // fixme align up
  footprint += n_slots * (sizeof( uint32_t )+sizeof( uint32_t ));

  fd = open( path, O_RDWR | O_CREAT | O_EXCL, 0600 );
  if( fd < 0 ) goto fail;

  ret = posix_fallocate( fd, 0, (off_t)footprint );
  if( ret != 0 ) goto fail;

  wtr = malloc( sizeof( *wtr ) );
  if( !wtr ) goto fail;

  wtr->footprint = footprint;
  wtr->log = mmap( NULL, footprint, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0 );
  if( wtr->log == MAP_FAILED ) goto fail;

  close( fd ); // failure just means a leak happens

  wtr->log->tail_idx       = 0;
  wtr->log->n_slots        = n_slots;
  wtr->log->entries_offset = strings_region_sz;

  char* buffer = log_fname_buffer( wtr->log );
  for( uint32_t i = 0; i < field_names_len; ++i ) {
    char const* name = field_names[i];
    size_t len = strlen( name );
    memcpy( buffer, name, len+1 );
    buffer += len;
  }

  log_entry_t* entries = log_entries( wtr->log );
  for( uint32_t i = 0; i < n_slots; ++i ) {
    entries[i].field = UNUSED_ENTRY;
  }

  wtr->log->magic = RB_LOG_MAGIC;

  return wtr;

fail:
  if( fd != -1 ) close( fd );
  return NULL;
}

void
delete_rb_log_writer( rb_log_writer_t * wtr )
{
  if( !wtr ) return;
  munmap( wtr->log, wtr->footprint );
  free( wtr );
}

void
rb_insert( rb_log_writer_t * wtr,
           uint32_t          field,
           uint32_t          value )
{
  // ASSUME( log ); // FIXME implement assume

  uint64_t next_tail = wtr->log->tail+1;
  if( next_tail > wtr->log->n_slots ) {
    next_tail = 0;
  }
}
