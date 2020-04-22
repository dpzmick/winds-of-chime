#pragma once

#include <stdint.h>

#define LIKELY(cond)   (__builtin_expect(!!(cond), 1))
#define UNLIKELY(cond) (__builtin_expect(!!(cond), 0))

#define ARRAY_SIZE(arr) ( sizeof( arr )/sizeof( *arr ) )

#define FATAL(...) fatal_impl( __FILE__, __LINE__, __VA_ARGS__ )

void
__attribute__((cold))
__attribute__((noreturn))
__attribute__((format(printf, 3, 4)))
fatal_impl( char const * file,
            int          line,
            char const*  fmt,
                         ... );

static inline uint64_t
rdtscp( void )
{
  uint32_t hi, lo;
  __asm__ volatile( "rdtscp": "=a"(lo), "=d"(hi));
  return (uint64_t)lo | ( (uint64_t)hi << 32 );
}
