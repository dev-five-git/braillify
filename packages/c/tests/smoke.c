#include "braillify.h"

#include <stdio.h>

static int print_or_fail(char *braille) {
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

int main(void) {
    if (print_or_fail(braillify_encode_unicode("안녕하세요")) != 0) {
        return 1;
    }
    return print_or_fail(braillify_encode_unicode_in_context("pOH", "science"));
}
