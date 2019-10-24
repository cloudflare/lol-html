#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

// Memory limiting
//---------------------------------------------------------------------
void test_memory_limiting() {
    const char *chunk1 = "<span alt='aaaaa";
    const int max_memory = 5;

    RUN_REWRITER_WITH_MAX_MEMORY(chunk1, output_sink_stub,
        {
            const char *selector_str = "span";
            cool_thing_selector_t *selector = cool_thing_selector_parse(
                selector_str,
                strlen(selector_str)
            );

            cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                &get_and_free_empty_element_attribute,
                NULL,
                NULL,
                NULL,
                NULL,
                NULL
            );
        },
        {
            ok(cool_thing_rewriter_write(rewriter, in, strlen(in)) == -1);

            cool_thing_str_t *msg = cool_thing_take_last_error();

            str_eq(msg, "The memory limit has been exceeded.");
            cool_thing_str_free(*msg);
        },
    max_memory);
}
