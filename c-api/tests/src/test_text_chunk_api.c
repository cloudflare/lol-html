#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static int EXPECTED_USER_DATA = 42;

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    insert_after_text_chunk_output_sink,
    "<div>Hey 42&lt;/div&gt;",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t insert_before_and_after_text_chunk(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(user_data);

    const char *before = "<div>";
    const char *after = "</div>";
    cool_thing_text_chunk_content_t content = cool_thing_text_chunk_content_get(chunk);

    if (content.len > 0) {
        note("Content");
        str_eq(&content, "Hey 42");

        note("Remove and last in text node flags");
        ok(!cool_thing_text_chunk_is_last_in_text_node(chunk));
        ok(!cool_thing_text_chunk_is_removed(chunk));

        note("Insert before after");
        ok(!cool_thing_text_chunk_before(chunk, before, strlen(before), true));
        ok(!cool_thing_text_chunk_after(chunk, after, strlen(after), false));
    } else {
        note("Last in text node flag for the last chunk");
        ok(cool_thing_text_chunk_is_last_in_text_node(chunk));
    }

    return COOL_THING_CONTINUE;
}

static void test_insert_before_and_after_text_chunk(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

     cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &insert_before_and_after_text_chunk,
        NULL
    );

    run_rewriter(builder, "Hey 42", insert_after_text_chunk_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    modify_user_data_output_sink,
    "Hey 42",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t modify_user_data(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    note("User data");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    note("Set text chunk user data");
    cool_thing_text_chunk_user_data_set(chunk, user_data);

    note("Get text chunk user data");

    int chunk_user_data = *(int*)cool_thing_text_chunk_user_data_get(chunk);

    ok(chunk_user_data == EXPECTED_USER_DATA);

    return COOL_THING_CONTINUE;
}

static void test_modify_user_data(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &modify_user_data,
        user_data
    );

    run_rewriter(builder, "Hey 42", modify_user_data_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    replace_chunk_output_sink,
    "<div><repl></div>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t replace_chunk(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(user_data);
    const char *replacement = "<repl>";

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Replace");
        ok(!cool_thing_text_chunk_replace(chunk, replacement, strlen(replacement), true));
        ok(cool_thing_text_chunk_is_removed(chunk));
    }

    return COOL_THING_CONTINUE;
}

static void test_replace_chunk(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        NULL,
        NULL,
        NULL,
        NULL,
        &replace_chunk,
        NULL
    );

    ok(!err);

    run_rewriter(builder, "<div>Hello</div>", replace_chunk_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    insert_after_chunk_output_sink,
    "<div>Hello<after></div>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t insert_after_chunk(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(user_data);
    const char *after = "<after>";

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Insert after replaced");
        ok(!cool_thing_text_chunk_after(chunk, after, strlen(after), true));
    }

    return COOL_THING_CONTINUE;
}

static void test_insert_after_chunk(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &insert_after_chunk,
        NULL
    );

    run_rewriter(builder, "<div>Hello</div>", insert_after_chunk_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    remove_chunk_output_sink,
    "<span></span>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t remove_chunk(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(user_data);

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Remove");
        cool_thing_text_chunk_remove(chunk);
        ok(cool_thing_text_chunk_is_removed(chunk));
    }

    return COOL_THING_CONTINUE;
}

static void test_remove_chunk(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &remove_chunk,
        NULL
    );

    run_rewriter(builder, "<span>0_0</span>", remove_chunk_output_sink, user_data);
}

//-------------------------------------------------------------------------
static cool_thing_rewriter_directive_t stop_rewriting(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(chunk);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

static void test_stop_with_selector(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        NULL,
        NULL,
        NULL,
        NULL,
        &stop_rewriting,
        NULL
    );

    ok(!err);
    expect_stop(builder, "<div>42</div>", user_data);
}

static void test_stop(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

     cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &stop_rewriting,
        NULL
    );

     expect_stop(builder, "42", user_data);
}

void test_text_chunk_api() {
    int user_data = 42;

    const char *selector_str = "*";

    cool_thing_selector_t *selector = cool_thing_selector_parse(
        selector_str,
        strlen(selector_str)
    );

    test_insert_before_and_after_text_chunk(&user_data);
    test_modify_user_data(&user_data);
    test_replace_chunk(selector, &user_data);
    test_insert_after_chunk(&user_data);
    test_remove_chunk(&user_data);

    test_stop_with_selector(selector, &user_data);
    test_stop(&user_data);

    cool_thing_selector_free(selector);
}
