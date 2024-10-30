#include "../../include/lol_html.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static int EXPECTED_USER_DATA = 43;

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    append_to_empty_doc_output_sink,
    "<!--appended text-->hello &amp; world",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t append_to_empty_doc(
    lol_html_doc_end_t *doc_end,
    void *user_data
) {
    note("Append at at the end of an empty document");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    const char *append_html = "<!--appended text-->";
    ok(!lol_html_doc_end_append(doc_end, append_html, strlen(append_html), true));

    const char *append_text = "hello & world";
    ok(!lol_html_doc_end_append(doc_end, append_text, strlen(append_text), false));

    return LOL_HTML_CONTINUE;
}

static void test_append_to_empty_doc(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        append_to_empty_doc,
        user_data
    );

    run_rewriter(
        builder,
        "",
        append_to_empty_doc_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    append_at_end_output_sink,
    "<html><div>Hello</div></html><!--appended text-->hello &amp; world",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t append_at_end(
    lol_html_doc_end_t *doc_end,
    void *user_data
) {
    note("Append at at the end");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    const char *append_html = "<!--appended text-->";
    ok(!lol_html_doc_end_append(doc_end, append_html, strlen(append_html), true));

    const char *append_text = "hello & world";
    ok(!lol_html_doc_end_append(doc_end, append_text, strlen(append_text), false));

    return LOL_HTML_CONTINUE;
}

static void test_append_at_end(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        append_at_end,
        user_data
    );

    run_rewriter(
        builder,
        "<html><div>Hello</div></html>",
        append_at_end_output_sink,
        user_data
    );
}

void document_end_api_test() {
    int user_data = 43;

    test_append_to_empty_doc(&user_data);
    test_append_at_end(&user_data);
}
