#include <string.h>

#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

void test_non_ascii_encoding() {
    const char *encoding = "UTF-16";
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_t *rewriter = cool_thing_rewriter_build(
        builder,
        encoding,
        strlen(encoding),
        (cool_thing_memory_settings_t) {
            .preallocated_parsing_buffer_size = 0,
            .max_allowed_memory_usage = 16
        },
        &output_sink_stub,
        NULL,
        true
    );

    cool_thing_rewriter_builder_free(builder);

    ok(rewriter == NULL);

    cool_thing_str_t *msg = cool_thing_take_last_error();

    str_eq(msg, "Expected ASCII-compatible encoding.");

    cool_thing_str_free(*msg);
}
