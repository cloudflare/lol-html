#include <stdlib.h>

#include "deps/picotest/picotest.h"
#include "test_util.h"

void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data) {
    UNUSED(chunk);
    UNUSED(chunk_len);
    UNUSED(user_data);
}

cool_thing_rewriter_directive_t get_and_free_empty_element_attribute(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *attr1 = "foo";

    note("Has attribute");
    ok(cool_thing_element_has_attribute(element, attr1, strlen(attr1)) == 1);

    note("Get attribute");
    cool_thing_str_t *value = cool_thing_element_get_attribute(
        element,
        attr1,
        strlen(attr1)
    );

    str_eq(value, "");
    cool_thing_str_free(*value);

    return COOL_THING_CONTINUE;
}


cool_thing_rewriter_t* create_rewriter(cool_thing_rewriter_builder_t *builder,
    output_sink_t output_sink,
    size_t max_memory
) {
    const char *encoding = "UTF-8";
    int output_sink_user_data = 42;

    cool_thing_rewriter_t *rewriter = cool_thing_rewriter_build(
        builder,
        encoding,
        strlen(encoding),
        (cool_thing_memory_settings_t) {
            .preallocated_parsing_buffer_size = 0,
            .max_allowed_memory_usage = max_memory
        },
        output_sink,
        &output_sink_user_data,
        true
    );

    cool_thing_rewriter_builder_free(builder);

    return rewriter;
}

void run_rewriter(cool_thing_rewriter_builder_t *builder,
    const char *html,
    output_sink_t output_sink
) {
    const char *in = html;
    cool_thing_rewriter_t *rewriter = create_rewriter(builder, output_sink, MAX_MEMORY);

    ok(!cool_thing_rewriter_write(rewriter, in, strlen(in)));
    ok(!cool_thing_rewriter_end(rewriter));

    cool_thing_rewriter_free(rewriter);
}

void expect_stop(cool_thing_rewriter_builder_t *builder, const char *html) {
    const char *in = html;
    cool_thing_rewriter_t *rewriter = create_rewriter(builder, output_sink_stub, MAX_MEMORY);

    ok(cool_thing_rewriter_write(rewriter, in, strlen(in)));
    cool_thing_str_t *msg = cool_thing_take_last_error();
    str_eq(msg, "The rewriter has been stopped.");
    cool_thing_str_free(*msg);
}

void check_output(const char *chunk,
    const char *expected,
    size_t start_idx,
    size_t bytes,
    void *user_data
) {
    ok(*(int*)user_data == 42);

    if (bytes > 0) {
        ok(!memcmp(chunk, expected + start_idx, bytes));
    } else {
        // The last chunk
        ok(start_idx == strlen(expected));
    }
}
