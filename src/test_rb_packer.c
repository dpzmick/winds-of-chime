#include "rb/packer.h"

#include <fcntl.h>
#include <unistd.h>

char const*
__attribute__((noinline)) pack( rb_packer_t * p )
{
  rb_packer_transaction_t t[] = { rb_packer_start( p ) };

  rb_packer_start_object( t, 100 );
  rb_packer_add_u32( t, 12 );
  rb_packer_add_u32( t, 13 );
  rb_packer_add_u32( t, 14 );
  rb_packer_add_u32( t, 15 );
  return rb_packer_finalize( *t );
}

int main() {
  rb_packer_t * p = new_rb_packer( NULL );
  char const * b = pack( p );

  int fd = open( "out", O_WRONLY | O_CREAT, 0600 );
  if( fd < 0 ) abort();

  write( fd, b, 4096 );
}
