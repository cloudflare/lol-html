#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

void test_memory_limiting() {
    const char *chunk1 = "<span alt='aaaaa";
    const int max_memory = 5;
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();
    cool_thing_rewriter_t *rewriter = NULL;

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

    rewriter = create_rewriter(builder, output_sink_stub, max_memory);

    ok(cool_thing_rewriter_write(rewriter, chunk1, strlen(chunk1)) == -1);

    cool_thing_str_t *msg = cool_thing_take_last_error();

    str_eq(msg, "The memory limit has been exceeded.");
    cool_thing_str_free(*msg);
}
