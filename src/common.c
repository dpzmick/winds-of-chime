#include "common.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

void
fatal_impl( char const* file,
            int         line,
            char const* fmt,
                        ... )
{
  va_list args;
  va_start(args, fmt);

  fprintf(stderr, "Fatal Error at %s: %d: ", file, line);
  vfprintf(stderr, fmt, args);
  fprintf(stderr, "\n");

  va_end(args);

  abort();
}
