#include "../../include/lol_html.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static int EXPECTED_USER_DATA = 42;

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_set_comment_text_output_sink,
    "<!--Yo-->",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t get_set_comment_text(
    lol_html_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *new_text = "Yo";

    note("Get/set text");
    lol_html_str_t text = lol_html_comment_text_get(comment);

    str_eq(&text, "Hey 42");

    lol_html_str_free(text);

    ok(!lol_html_comment_text_set(comment, new_text, strlen(new_text)));

    return LOL_HTML_CONTINUE;
}


static void test_get_set_comment_text(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &get_set_comment_text,
        user_data,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(builder, "<!--Hey 42-->", get_set_comment_text_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    insert_before_and_after_comment_output_sink,
    "<div><!--Hey 42-->&lt;/div&gt;",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t insert_before_and_after_comment(
    lol_html_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *before = "<div>";
    const char *after = "</div>";

    note("Insert before/after");
    ok(!lol_html_comment_before(comment, before, strlen(before), true));
    ok(!lol_html_comment_after(comment, after, strlen(after), false));

    return LOL_HTML_CONTINUE;
}

static void test_insert_before_and_after_comment(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &insert_before_and_after_comment,
        user_data,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(
        builder,
        "<!--Hey 42-->",
        insert_before_and_after_comment_output_sink,
        user_data
    );
}


//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_set_user_data_output_sink,
    "<!--33-->",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t get_set_user_data(
    lol_html_comment_t *comment,
    void *user_data
) {
    note("Set comment user data");
    lol_html_comment_user_data_set(comment, user_data);

    note("User data");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    note("Get comment user data");
    int comment_user_data = *(int*)lol_html_comment_user_data_get(comment);
    ok(comment_user_data == EXPECTED_USER_DATA);

    return LOL_HTML_CONTINUE;
}

static void test_get_set_user_data(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &get_set_user_data,
        user_data,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(builder, "<!--33-->", get_set_user_data_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    replace_comment_output_sink,
    "<div><repl></div>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t replace_comment(
    lol_html_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *replacement = "<repl>";

    note("Replace");
    ok(!lol_html_comment_replace(comment, replacement, strlen(replacement), true));
    ok(lol_html_comment_is_removed(comment));

    return LOL_HTML_CONTINUE;
}

static void test_replace_comment(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        NULL,
        NULL,
        &replace_comment,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(builder, "<div><!--hello--></div>", replace_comment_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    insert_after_comment_output_sink,
    "<div><!--hello--><after></div>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t insert_after_comment(
    lol_html_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *after = "<after>";

    note("Insert after comment");
    ok(!lol_html_comment_after(comment, after, strlen(after), true));

    return LOL_HTML_CONTINUE;
}

static void test_insert_after_comment(lol_html_selector_t *selector, void *user_data) {
    UNUSED(selector);

    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &insert_after_comment,
        NULL,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(
        builder,
        "<div><!--hello--></div>",
        insert_after_comment_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    remove_comment_output_sink,
    "<>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static lol_html_rewriter_directive_t remove_comment(
    lol_html_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    note("Removed flag");
    ok(!lol_html_comment_is_removed(comment));

    note("Remove");
    lol_html_comment_remove(comment);
    ok(lol_html_comment_is_removed(comment));

    return LOL_HTML_CONTINUE;
}

static void test_remove_comment(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

     lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &remove_comment,
        NULL,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(builder, "<<!--0_0-->>", remove_comment_output_sink, user_data);
}

//-------------------------------------------------------------------------
static lol_html_rewriter_directive_t stop_rewriting(
    lol_html_comment_t *comment,
    void *user_data
) {
    UNUSED(comment);
    UNUSED(user_data);

    note("Stop rewriting");

    return LOL_HTML_STOP;
}

static void test_stop(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &stop_rewriting,
        NULL,
        NULL,
        NULL,
        NULL
    );

    expect_stop(builder, "<!-- hey -->", user_data);
}

//-------------------------------------------------------------------------
static void test_stop_with_selector(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        NULL,
        NULL,
        &stop_rewriting,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    expect_stop(builder, "<div><!-- foo --></div>", user_data);
}

void test_comment_api() {
    int user_data = 42;

    const char *selector_str = "*";

    lol_html_selector_t *selector = lol_html_selector_parse(
        selector_str,
        strlen(selector_str)
    );

    test_get_set_comment_text(&user_data);
    test_get_set_user_data(&user_data);
    test_replace_comment(selector, &user_data);
    test_insert_after_comment(selector, &user_data);
    test_remove_comment(&user_data);
    test_insert_before_and_after_comment(&user_data);

    test_stop(&user_data);
    test_stop_with_selector(selector, &user_data);

    lol_html_selector_free(selector);
}
