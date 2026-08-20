#include "braillify.h"

#include <stdio.h>

int main(void) {
    char *braille = braillify_encode_unicode("안녕하세요");
    if (braille == NULL) {
        char *error = braillify_last_error();
        fprintf(stderr, "braillify: %s\n", error != NULL ? error : "unknown error");
        braillify_string_free(error);
        return 1;
    }

    puts(braille);
    braillify_string_free(braille);
    return 0;
}

