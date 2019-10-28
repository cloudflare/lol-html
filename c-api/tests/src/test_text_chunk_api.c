#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler1(
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

cool_thing_rewriter_directive_t test_text_chunk_api_user_data_get(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(user_data);

    note("Get text chunk user data");

    int chunk_user_data = *(int*)cool_thing_text_chunk_user_data_get(chunk);

    ok(chunk_user_data == 42);

    return COOL_THING_CONTINUE;
}

cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler2_el(
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

cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler2_doc(
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

cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler3(
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

cool_thing_rewriter_directive_t test_text_chunk_api_stop_rewriting(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    UNUSED(chunk);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

EXPECT_OUTPUT(
    test_text_chunk_api_output1,
    "<div>Hey 42&lt;/div&gt;"
);

EXPECT_OUTPUT(
    test_text_chunk_api_output2,
    "<div><repl><after></div>"
);

EXPECT_OUTPUT(
    test_text_chunk_api_output3,
    "<span></span>"
);

void test_text_chunk_api() {
    int user_data = 42;

    const char *selector_str = "*";

    cool_thing_selector_t *selector = cool_thing_selector_parse(
        selector_str,
        strlen(selector_str)
    );

    REWRITE(
        "Hey 42",
        test_text_chunk_api_output1,
        {
             cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                NULL,
                NULL,
                &test_text_chunk_api_text_chunk_handler1,
                &user_data
            );

            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                NULL,
                NULL,
                &test_text_chunk_api_user_data_get,
                NULL
            );
        }
    );

    REWRITE(
        "<div>Hello</div>",
        test_text_chunk_api_output2,
        {
            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                NULL,
                NULL,
                NULL,
                NULL,
                &test_text_chunk_api_text_chunk_handler2_el,
                NULL
            );

            ok(!err);

            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                NULL,
                NULL,
                &test_text_chunk_api_text_chunk_handler2_doc,
                NULL
            );
        }
    );

    REWRITE(
        "<span>0_0</span>",
        test_text_chunk_api_output3,
        {
            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                NULL,
                NULL,
                &test_text_chunk_api_text_chunk_handler3,
                NULL
            );
        }
    );

    EXPECT_STOP(
        "<div>42</div>",
        {
            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                NULL,
                NULL,
                NULL,
                NULL,
                &test_text_chunk_api_stop_rewriting,
                NULL
            );

            ok(!err);
        }
    );

    EXPECT_STOP(
        "42",
        {
             cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                NULL,
                NULL,
                &test_text_chunk_api_stop_rewriting,
                NULL
            );
        }
    );

    cool_thing_selector_free(selector);
}
