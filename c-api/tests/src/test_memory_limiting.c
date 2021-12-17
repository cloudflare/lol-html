#include "../../include/lol_html.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

void test_memory_limiting() {
    const char *chunk1 = "<span alt='aaaaa";
    const int max_memory = 5;
    int user_data = 42;
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();
    lol_html_rewriter_t *rewriter = NULL;

    const char *selector_str = "span";
    lol_html_selector_t *selector = lol_html_selector_parse(
        selector_str,
        strlen(selector_str)
    );

    lol_html_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &get_and_free_empty_element_attribute,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    rewriter = create_rewriter(builder, output_sink_stub, &user_data, max_memory);

    ok(lol_html_rewriter_write(rewriter, chunk1, strlen(chunk1)) == -1);

    lol_html_str_t *msg = lol_html_take_last_error();

    str_eq(msg, "The memory limit has been exceeded.");
    lol_html_str_free(*msg);
    lol_html_rewriter_free(rewriter);
    lol_html_selector_free(selector);
}
