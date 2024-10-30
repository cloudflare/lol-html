#include <string.h>

#include "../../include/lol_html.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

void test_non_ascii_encoding() {
    const char *encoding = "UTF-16";
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_t *rewriter = lol_html_rewriter_build(
        builder,
        encoding,
        strlen(encoding),
        (lol_html_memory_settings_t) {
            .preallocated_parsing_buffer_size = 0,
            .max_allowed_memory_usage = 16
        },
        &output_sink_stub,
        NULL,
        true
    );

    lol_html_rewriter_builder_free(builder);

    ok(rewriter == NULL);

    lol_html_str_t msg = lol_html_take_last_error();

    str_eq(msg, "Expected ASCII-compatible encoding.");

    lol_html_str_free(msg);
}
