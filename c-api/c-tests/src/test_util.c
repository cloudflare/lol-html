#include <stdio.h>
#include <stdlib.h>
#include <assert.h>

#include "deps/picotest/picotest.h"
#include "test_util.h"

void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data) {
    UNUSED(chunk);
    UNUSED(chunk_len);
    UNUSED(user_data);
}

lol_html_rewriter_directive_t get_and_free_empty_element_attribute(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *attr1 = "foo";

    note("Has attribute");
    ok(lol_html_element_has_attribute(element, attr1, strlen(attr1)) == 1);

    note("Get attribute");
    lol_html_str_t value = lol_html_element_get_attribute(
        element,
        attr1,
        strlen(attr1)
    );

    str_eq(value, "");
    lol_html_str_free(value);

    return LOL_HTML_CONTINUE;
}

lol_html_rewriter_t* create_rewriter(
    lol_html_rewriter_builder_t *builder,
    output_sink_t output_sink,
    void *output_sink_user_data,
    size_t max_memory
) {
    const char *encoding = "UTF-8";

    lol_html_rewriter_t *rewriter = lol_html_rewriter_build(
        builder,
        encoding,
        strlen(encoding),
        (lol_html_memory_settings_t) {
            .preallocated_parsing_buffer_size = 0,
            .max_allowed_memory_usage = max_memory
        },
        output_sink,
        output_sink_user_data,
        true
    );

    lol_html_rewriter_builder_free(builder);

    return rewriter;
}

void run_rewriter(
    lol_html_rewriter_builder_t *builder,
    const char *html,
    output_sink_t output_sink,
    void *output_sink_user_data
) {
    const char *in = html;
    lol_html_rewriter_t *rewriter = create_rewriter(
        builder,
        output_sink,
        output_sink_user_data,
        MAX_MEMORY
    );

    ok(!lol_html_rewriter_write(rewriter, in, strlen(in)));
    ok(!lol_html_rewriter_end(rewriter));

    lol_html_rewriter_free(rewriter);
}

void expect_stop(lol_html_rewriter_builder_t *builder, const char *html, void *user_data) {
    const char *in = html;
    lol_html_rewriter_t *rewriter = create_rewriter(
        builder,
        output_sink_stub,
        user_data,
        MAX_MEMORY
    );

    ok(lol_html_rewriter_write(rewriter, in, strlen(in)));
    lol_html_str_t msg = lol_html_take_last_error();
    str_eq(msg, "The rewriter has been stopped.");
    lol_html_str_free(msg);
    lol_html_rewriter_free(rewriter);
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
        int same_len = *out_len == strlen(expected);
        ok(same_len);
        int same_data = !memcmp(*out, expected, *out_len);
        ok(same_data);
        if (!same_len || !same_data) {
            printf("err: '%s' != '%s'\n", *out, expected);
        }
    }
}

void check_user_data(void *user_data, void *user_data_expected, size_t user_data_len) {
    assert(user_data != NULL);
    ok(!memcmp(user_data, user_data_expected, user_data_len));
}
