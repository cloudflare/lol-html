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

cool_thing_rewriter_t* create_rewriter(
    cool_thing_rewriter_builder_t *builder,
    output_sink_t output_sink,
    void *output_sink_user_data,
    size_t max_memory
) {
    const char *encoding = "UTF-8";

    cool_thing_rewriter_t *rewriter = cool_thing_rewriter_build(
        builder,
        encoding,
        strlen(encoding),
        (cool_thing_memory_settings_t) {
            .preallocated_parsing_buffer_size = 0,
            .max_allowed_memory_usage = max_memory
        },
        output_sink,
        output_sink_user_data,
        true
    );

    cool_thing_rewriter_builder_free(builder);

    return rewriter;
}

void run_rewriter(
    cool_thing_rewriter_builder_t *builder,
    const char *html,
    output_sink_t output_sink,
    void *output_sink_user_data
) {
    const char *in = html;
    cool_thing_rewriter_t *rewriter = create_rewriter(
        builder,
        output_sink,
        output_sink_user_data,
        MAX_MEMORY
    );

    ok(!cool_thing_rewriter_write(rewriter, in, strlen(in)));
    ok(!cool_thing_rewriter_end(rewriter));

    cool_thing_rewriter_free(rewriter);
}

void expect_stop(cool_thing_rewriter_builder_t *builder, const char *html, void *user_data) {
    const char *in = html;
    cool_thing_rewriter_t *rewriter = create_rewriter(
        builder,
        output_sink_stub,
        user_data,
        MAX_MEMORY
    );

    ok(cool_thing_rewriter_write(rewriter, in, strlen(in)));
    cool_thing_str_t *msg = cool_thing_take_last_error();
    str_eq(msg, "The rewriter has been stopped.");
    cool_thing_str_free(*msg);
}

void check_output(
    char **out,
    size_t *out_len,
    const char *chunk,
    size_t chunk_len,
    const char *expected
) {
    if (chunk_len > 0) {
        *out = (char *) (out == NULL ? malloc(chunk_len) : realloc(*out, *out_len + chunk_len));
        memcpy(*out + *out_len, chunk, chunk_len);
        *out_len += chunk_len;
    } else {
        ok(*out_len == strlen(expected));
        ok(!memcmp(*out, expected, *out_len));
    }
}

void check_user_data(void *user_data, void *user_data_expected, size_t user_data_len) {
    ok(!memcmp(user_data, user_data_expected, user_data_len));
}
