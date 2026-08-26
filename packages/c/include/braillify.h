#ifndef BRAILLIFY_H
#define BRAILLIFY_H

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32) && defined(BRAILLIFY_SHARED)
#  if defined(BRAILLIFY_BUILD)
#    define BRAILLIFY_API __declspec(dllexport)
#  else
#    define BRAILLIFY_API __declspec(dllimport)
#  endif
#else
#  define BRAILLIFY_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

/*
 * All input strings must be NUL-terminated UTF-8. Returned pointers are owned
 * by the caller and must be released with the matching function below.
 * On failure, encoding functions return NULL and braillify_last_error()
 * returns a newly allocated description for the calling thread.
 */

BRAILLIFY_API uint8_t *braillify_encode(const char *text, size_t *out_len);
BRAILLIFY_API char *braillify_encode_unicode(const char *text);
BRAILLIFY_API char *braillify_encode_braille_font(const char *text);
BRAILLIFY_API char *braillify_last_error(void);

BRAILLIFY_API void braillify_bytes_free(uint8_t *bytes, size_t len);
BRAILLIFY_API void braillify_string_free(char *value);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* BRAILLIFY_H */

