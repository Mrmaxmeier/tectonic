/* tectonic/tectonic.h: global, public header for Tectonic
   Copyright 2016 the Tectonic Project
   Licensed under the MIT License.
*/

#ifndef TECTONIC_TECTONIC_H
#define TECTONIC_TECTONIC_H

#include "bindings.h"
#include <stdbool.h>
#include <stdint.h> /* for int32_t */

/* Convenience for C++: this way Emacs doesn't try to indent the prototypes,
 * which I find annoying. */

#ifdef __cplusplus
#define BEGIN_EXTERN_C extern "C" {
#define END_EXTERN_C }
#else
#define BEGIN_EXTERN_C
#define END_EXTERN_C
#endif

/* NORETURN portability */

#if defined __GNUC__ && __GNUC__  >= 3
#define NORETURN __attribute__((__noreturn__))
#else
#define NORETURN
#endif

/* Ditto for printf argument checking */

#if defined __GNUC__ && __GNUC__  >= 3
#define PRINTF_FUNC(ifmt,iarg) __attribute__((format(printf, ifmt, iarg)))
#else
#define PRINTF_FUNC(ifmt,iarg)
#endif

/* "integer" 32-bit integer type used frequently */

typedef int32_t integer;
#define INTEGER_MAX INT32_MAX
#define INTEGER_MIN INT32_MIN

/* TeX-specific enums */

typedef enum {
    HISTORY_SPOTLESS = 0,
    HISTORY_WARNING_ISSUED = 1,
    HISTORY_ERROR_ISSUED = 2,
    HISTORY_FATAL_ERROR = 3
} tt_history_t;

/* The actual API */

#include "constants.h"
#include "stringpool.h"

#endif /* not TECTONIC_TECTONIC_H */
