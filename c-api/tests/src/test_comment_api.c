#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static cool_thing_rewriter_directive_t handle_comment(
    cool_thing_comment_t *comment,
    void *user_data
) {
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

    note("User data");
    ok(*(int*)user_data == 42);

    note("Set comment user data");
    cool_thing_comment_user_data_set(comment, user_data);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t handle_el(
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

static cool_thing_rewriter_directive_t handle_doc(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    const char *after = "<after>";

    note("Insert after replaced");
    ok(!cool_thing_comment_after(comment, after, strlen(after), true));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t handle_remove(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    note("Remove");
    cool_thing_comment_remove(comment);
    ok(cool_thing_comment_is_removed(comment));

    return COOL_THING_CONTINUE;
}


static cool_thing_rewriter_directive_t user_data_get(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(user_data);

    note("Get comment user data");

    int comment_user_data = *(int*)cool_thing_comment_user_data_get(comment);

    ok(comment_user_data == 42);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t stop_rewriting(
    cool_thing_comment_t *comment,
    void *user_data
) {
    UNUSED(comment);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

EXPECT_OUTPUT(
    output_sink1,
    "<div><!--Yo-->&lt;/div&gt;"
);

static void test_output1(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &handle_comment,
        user_data,
        NULL,
        NULL
    );

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &user_data_get,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(builder, "<!--Hey 42-->", output_sink1, user_data);
}

EXPECT_OUTPUT(
    output_sink2,
    "<div><repl><after></div>"
);

static void test_output2(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        NULL,
        NULL,
        &handle_el,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &handle_doc,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(builder, "<div><!--hello--></div>", output_sink2, user_data);
}

EXPECT_OUTPUT(
    output_sink3,
    "<>"
);

static void test_output3(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

     cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        &handle_remove,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(builder, "<<!--0_0-->>", output_sink3, user_data);
}

static void test_stop1(void *user_data) {
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

static void test_stop2(cool_thing_selector_t *selector, void *user_data) {
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

    test_output1(&user_data);
    test_output2(selector, &user_data);
    test_output3(&user_data);

    test_stop1(&user_data);
    test_stop2(selector, &user_data);

    cool_thing_selector_free(selector);
}
