#ifndef TEST_UTIL_H
#define TEST_UTIL_H

#include <stdlib.h>
#include <string.h>

#include "../../include/cool_thing.h"

#define MAX_MEMORY 2048

#define EXPECT_OUTPUT(sink_name, expected) \
    static void sink_name(const char *chunk, size_t chunk_len, void *user_data) { \
        static size_t start_idx = 0; \
    \
        check_output(chunk, expected, start_idx, chunk_len, user_data); \
        start_idx += chunk_len; \
    }

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

#define RUN_REWRITER_WITH_MAX_MEMORY(html, output_sink, assign_handlers, actions, max_memory) \
    do { \
        const char *in = html; \
        const char *encoding = "UTF-8"; \
        int output_sink_user_data = 42; \
        cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new(); \
    \
        assign_handlers \
    \
        cool_thing_rewriter_t *rewriter = cool_thing_rewriter_build( \
            builder, \
            encoding, \
            strlen(encoding), \
            (cool_thing_memory_settings_t) { \
                .preallocated_parsing_buffer_size = 0, \
                .max_allowed_memory_usage = max_memory \
            }, \
            &output_sink, \
            &output_sink_user_data, \
            true \
        ); \
    \
        cool_thing_rewriter_builder_free(builder); \
        actions \
        cool_thing_rewriter_free(rewriter); \
    } while(0)

#define RUN_REWRITER(html, output_sink, assign_handlers, actions) \
    RUN_REWRITER_WITH_MAX_MEMORY(html, output_sink, assign_handlers, actions, 2048)

#define REWRITE(html, output_sink, assign_handlers) \
    RUN_REWRITER(html, output_sink, assign_handlers, { \
        ok(!cool_thing_rewriter_write(rewriter, in, strlen(in))); \
        ok(!cool_thing_rewriter_end(rewriter)); \
    })

#define EXPECT_STOP(html, assign_handlers) \
    RUN_REWRITER(html, output_sink_stub, assign_handlers, { \
        ok(cool_thing_rewriter_write(rewriter, in, strlen(in))); \
    \
        cool_thing_str_t *msg = cool_thing_take_last_error(); \
    \
        str_eq(msg, "The rewriter has been stopped."); \
    \
        cool_thing_str_free(*msg); \
    })

#define str_eq(actual, expected) { \
    ok((actual) != NULL); \
    ok((actual)->len == strlen(expected)); \
    ok(!memcmp((actual)->data, expected, (actual)->len)); \
}

#define c_str_eq(actual, expected) ok(!strcmp(actual, expected))

#define UNUSED (void)

void check_output(const char *chunk,
    const char *expected,
    size_t start_idx,
    size_t bytes,
    void *user_data
);

void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data);

cool_thing_rewriter_directive_t get_and_free_empty_element_attribute(
    cool_thing_element_t *element,
    void *user_data
);

#endif // TEST_UTIL_H
