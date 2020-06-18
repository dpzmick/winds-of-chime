#pragma once

#include <assert.h>

// notes:
// - https://gcc.gnu.org/onlinedocs/gcc/Statement-Exprs.html
//
// usage:
// char * x = ASSUME(get_x());

#ifndef NDEBUG
#define ASSUME( cond ) { assert( (cond) ); (cond) }
#else
#define ASSUME( cond ) { __builtin_assume(!!(cond)); (cond) }
#endif
