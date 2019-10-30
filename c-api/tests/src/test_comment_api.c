#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static int EXPECTED_USER_DATA = 42;

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_set_comment_text_output_sink,
    "<div><!--Yo-->&lt;/div&gt;",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t get_set_comment_text(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *before = "<div>";
    const char *after = "</div>";
    const char *new_text = "Yo";

    note("Get/set text");
    cool_thing_str_t text = cool_thing_comment_text_get(comment);

    str_eq(&text, "Hey 42");

    cool_thing_str_free(text);

    ok(!cool_thing_comment_text_set(comment, new_text, strlen(new_text)));

    note("Removed flag");
    ok(!cool_thing_comment_is_removed(comment));

    note("Insert before/after");
    ok(!cool_thing_comment_before(comment, before, strlen(before), true));
    ok(!cool_thing_comment_after(comment, after, strlen(after), false));

    return COOL_THING_CONTINUE;
}

static void test_get_set_comment_text(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &get_set_comment_text,
        user_data,
        NULL,
        NULL
    );

    run_rewriter(builder, "<!--Hey 42-->", get_set_comment_text_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_set_user_data_output_sink,
    "<!--33-->",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t get_set_user_data(
    cool_thing_comment_t *comment,
    void *user_data
) {
    note("Set comment user data");
    cool_thing_comment_user_data_set(comment, user_data);

    note("User data");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    note("Get comment user data");
    int comment_user_data = *(int*)cool_thing_comment_user_data_get(comment);
    ok(comment_user_data == EXPECTED_USER_DATA);

    return COOL_THING_CONTINUE;
}

static void test_get_set_user_data(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &get_set_user_data,
        user_data,
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

static cool_thing_rewriter_directive_t replace_comment(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *replacement = "<repl>";

    note("Replace");
    ok(!cool_thing_comment_replace(comment, replacement, strlen(replacement), true));
    ok(cool_thing_comment_is_removed(comment));

    return COOL_THING_CONTINUE;
}

static void test_replace_comment(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
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

static cool_thing_rewriter_directive_t insert_after_comment(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *after = "<after>";

    note("Insert after comment");
    ok(!cool_thing_comment_after(comment, after, strlen(after), true));

    return COOL_THING_CONTINUE;
}

static void test_insert_after_comment(cool_thing_selector_t *selector, void *user_data) {
    UNUSED(selector);

    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &insert_after_comment,
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

static cool_thing_rewriter_directive_t remove_comment(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    note("Remove");
    cool_thing_comment_remove(comment);
    ok(cool_thing_comment_is_removed(comment));

    return COOL_THING_CONTINUE;
}

static void test_remove_comment(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

     cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &remove_comment,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(builder, "<<!--0_0-->>", remove_comment_output_sink, user_data);
}

//-------------------------------------------------------------------------
static cool_thing_rewriter_directive_t stop_rewriting(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(comment);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

static void test_stop(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &stop_rewriting,
        NULL,
        NULL,
        NULL
    );

    expect_stop(builder, "<!-- hey -->", user_data);
}

//-------------------------------------------------------------------------
static void test_stop_with_selector(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
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

    cool_thing_selector_t *selector = cool_thing_selector_parse(
        selector_str,
        strlen(selector_str)
    );

    test_get_set_comment_text(&user_data);
    test_get_set_user_data(&user_data);
    test_replace_comment(selector, &user_data);
    test_insert_after_comment(selector, &user_data);
    test_remove_comment(&user_data);

    test_stop(&user_data);
    test_stop_with_selector(selector, &user_data);

    cool_thing_selector_free(selector);
}
