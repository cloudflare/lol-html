#ifndef TEST_UTIL_H
#define TEST_UTIL_H

#include <stdlib.h>
#include <string.h>

#include "../../include/lol_html.h"

#define MAX_MEMORY 2048

#define str_eq(actual, expected) { \
    ok((actual) != NULL); \
    ok((actual)->len == strlen(expected)); \
    ok(!memcmp((actual)->data, expected, (actual)->len)); \
}

#define c_str_eq(actual, expected) ok(!strcmp(actual, expected))

#define UNUSED (void)

#define EXPECT_OUTPUT(sink_name, out_expected, user_data_expected, user_data_len) \
    static void sink_name(const char *chunk, size_t chunk_len, void *user_data) { \
        static char *out = NULL; \
        static size_t out_len = 0; \
    \
        check_output(&out, &out_len, chunk, chunk_len, out_expected); \
        check_user_data(user_data, user_data_expected, user_data_len); \
    }

typedef void (*output_sink_t)(const char *, size_t, void *);

void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data);

lol_html_rewriter_directive_t get_and_free_empty_element_attribute(
    lol_html_element_t *element,
    void *user_data
);

lol_html_rewriter_t* create_rewriter(
    lol_html_rewriter_builder_t *builder,
    output_sink_t output_sink,
    void *output_sink_user_data,
    size_t max_memory
);

void run_rewriter(
    lol_html_rewriter_builder_t *builder,
    const char *html,
    output_sink_t output_sink,
    void *output_sink_user_data
);

void expect_stop(lol_html_rewriter_builder_t *builder, const char *html, void *user_data);

// If `chunk_len` is greater than 0, this concatenates `chunk_len` bytes from
// `chunk` to the string pointed to by `out`. Otherwise, it checks if the string
// pointer to by `out` is identical to `expected`.
void check_output(
    char **out,
    size_t *out_len,
    const char *chunk,
    size_t chunk_len,
    const char *expected
);

// Check if the first `user_data_len` bytes pointed to by `user_data` are the same as those
// `user_data_expected` points to.
//
// NOTE: Only the first `user_data_len` bytes are compared. If `user_data points` to a
// sequence of bytes which is longer than `user_data_len` bytes, this check
// will succeed even if the remaining bytes are not the same.
void check_user_data(void *user_data, void *user_data_expected, size_t user_data_len);

#endif // TEST_UTIL_H
