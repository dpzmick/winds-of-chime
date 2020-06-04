#pragma once

typedef enum {
  LOG_LEVEL_INFO,
  LOG_LEVEL_ERROR,
} log_level_t;

/* logs in the fast path for now */

#define LOG_INFO(...)  log_impl( __FILE__ + __FILE_HEADER_LEN__, __LINE__, LOG_LEVEL_INFO, __VA_ARGS__ )
#define LOG_ERROR(...) log_impl( __FILE__ + __FILE_HEADER_LEN__, __LINE__, LOG_LEVEL_ERROR, __VA_ARGS__ )

/* Can be called form any thread */

void
__attribute__((format(printf, 4, 5)))
log_impl( char const * file,
          int          line,
          log_level_t  level,
          char const*  fmt,
          ... );
