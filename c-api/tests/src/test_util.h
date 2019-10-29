#ifndef TEST_UTIL_H
#define TEST_UTIL_H

#include <stdlib.h>
#include <string.h>

#include "../../include/cool_thing.h"

#define MAX_MEMORY 2048

#define str_eq(actual, expected) { \
    ok((actual) != NULL); \
    ok((actual)->len == strlen(expected)); \
    ok(!memcmp((actual)->data, expected, (actual)->len)); \
}

#define c_str_eq(actual, expected) ok(!strcmp(actual, expected))

#define UNUSED (void)

#define EXPECT_OUTPUT(sink_name, expected) \
    static void sink_name(const char *chunk, size_t chunk_len, void *user_data) { \
        static char *out = NULL; \
        static size_t out_len = 0; \
    \
        check_output(&out, &out_len, chunk, chunk_len, expected, user_data); \
    }

void check_output(char **out,
    size_t *out_len,
    const char *chunk,
    size_t chunk_len,
    const char *expected,
    void *user_data
);

typedef void (*output_sink_t)(const char *, size_t, void *);

cool_thing_rewriter_t* create_rewriter(cool_thing_rewriter_builder_t *builder,
    output_sink_t output_sink,
    size_t max_memory
);

void run_rewriter(cool_thing_rewriter_builder_t *builder,
    const char *html,
    output_sink_t output_sink
);

void expect_stop(cool_thing_rewriter_builder_t *builder, const char *html);

void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data);

cool_thing_rewriter_directive_t get_and_free_empty_element_attribute(
    cool_thing_element_t *element,
    void *user_data
);

#endif // TEST_UTIL_H
