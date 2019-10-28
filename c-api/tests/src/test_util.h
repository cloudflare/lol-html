#ifndef TEST_UTIL_H
#define TEST_UTIL_H

#include <stdlib.h>
#include <string.h>

#include "../../include/cool_thing.h"

#define EXPECT_OUTPUT(sink_name, expected) \
    static void sink_name(const char *chunk, size_t chunk_len, void *user_data) { \
        static char *out = NULL; \
        static size_t out_len = 0; \
    \
        ok(*(int*)user_data == 42); \
    \
        if (chunk_len > 0) { \
            out = (char *) (out == NULL ? malloc(chunk_len) : realloc(out, out_len + chunk_len)); \
            memcpy(out + out_len, chunk, chunk_len); \
            out_len += chunk_len; \
        } else { \
            ok(out_len == strlen(expected)); \
            ok(!memcmp(out, expected, out_len)); \
        } \
    }

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

void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data);

cool_thing_rewriter_directive_t get_and_free_empty_element_attribute(
    cool_thing_element_t *element,
    void *user_data
);

#endif // TEST_UTIL_H
