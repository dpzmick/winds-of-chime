#include "log.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <threads.h>

void
log_impl( char const* file,
          int         line,
          log_level_t level,
          char const* fmt,
                        ... )
{
  va_list args;
  va_start(args, fmt);

  char const * hdr = NULL;
  switch (level) {
    case LOG_LEVEL_INFO:  hdr = "INFO"; break;
    case LOG_LEVEL_ERROR: hdr = "ERROR"; break;
  }

  time_t timer;
  char tm[26];
  struct tm* tm_info;

  timer = time(NULL);
  tm_info = localtime(&timer);

  strftime(tm, 26, "%Y-%m-%d %H:%M:%S", tm_info);

  fprintf(stderr, "%-5s %s %s:%-4d: ", hdr, tm, file, line);
  vfprintf(stderr, fmt, args);
  fprintf(stderr, "\n");

  va_end(args);
}
