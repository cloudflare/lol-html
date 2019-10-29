#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static cool_thing_rewriter_directive_t handle_chunk1(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
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

    note("User data");
    ok(*(int*)user_data == 42);

    note("Set text chunk user data");
    cool_thing_text_chunk_user_data_set(chunk, user_data);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t user_data_get(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(user_data);

    note("Get text chunk user data");

    int chunk_user_data = *(int*)cool_thing_text_chunk_user_data_get(chunk);

    ok(chunk_user_data == 42);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t handle_el(
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

static cool_thing_rewriter_directive_t handle_doc(
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

static cool_thing_rewriter_directive_t handle_chunk2(
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

static cool_thing_rewriter_directive_t stop_rewriting(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(chunk);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

EXPECT_OUTPUT(
    output_sink1,
    "<div>Hey 42&lt;/div&gt;"
);

static void test_output1(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

     cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &handle_chunk1,
        user_data
    );

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &user_data_get,
        NULL
    );

    run_rewriter(builder, "Hey 42", output_sink1);
}

EXPECT_OUTPUT(
    output_sink2,
    "<div><repl><after></div>"
);

static void test_output2(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        NULL,
        NULL,
        NULL,
        NULL,
        &handle_el,
        NULL
    );

    ok(!err);

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &handle_doc,
        NULL
    );

    run_rewriter(builder, "<div>Hello</div>", output_sink2);
}

EXPECT_OUTPUT(
    output_sink3,
    "<span></span>"
);

static void test_output3() {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        NULL,
        NULL,
        NULL,
        NULL,
        &handle_chunk2,
        NULL
    );

    run_rewriter(builder, "<span>0_0</span>", output_sink3);
}

static void test_stop1(cool_thing_selector_t *selector) {
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
    expect_stop(builder, "<div>42</div>");
}

static void test_stop2() {
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

     expect_stop(builder, "42");
}

void test_text_chunk_api() {
    int user_data = 42;

    const char *selector_str = "*";

    cool_thing_selector_t *selector = cool_thing_selector_parse(
        selector_str,
        strlen(selector_str)
    );

    test_output1(&user_data);
    test_output2(selector);
    test_output3();

    test_stop1(selector);
    test_stop2();

    cool_thing_selector_free(selector);
}
