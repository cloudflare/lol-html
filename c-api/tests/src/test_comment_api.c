#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

// Comment API
//---------------------------------------------------------------------
cool_thing_rewriter_directive_t test_comment_api_comment_handler1(
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

cool_thing_rewriter_directive_t test_comment_api_user_data_get(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(user_data);

    note("Get comment user data");

    int comment_user_data = *(int*)cool_thing_comment_user_data_get(comment);

    ok(comment_user_data == 42);

    return COOL_THING_CONTINUE;
}

cool_thing_rewriter_directive_t test_comment_api_comment_handler2_el(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(user_data);

    const char *replacement = "<repl>";

    note("Replace");
    ok(!cool_thing_comment_replace(comment, replacement, strlen(replacement), true));
    ok(cool_thing_comment_is_removed(comment));

    return COOL_THING_CONTINUE;
}

cool_thing_rewriter_directive_t test_comment_api_comment_handler2_doc(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(user_data);

    const char *after = "<after>";

    note("Insert after replaced");
    ok(!cool_thing_comment_after(comment, after, strlen(after), true));

    return COOL_THING_CONTINUE;
}

cool_thing_rewriter_directive_t test_comment_api_comment_handler3(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(user_data);

    note("Remove");
    cool_thing_comment_remove(comment);
    ok(cool_thing_comment_is_removed(comment));

    return COOL_THING_CONTINUE;
}

cool_thing_rewriter_directive_t test_comment_api_stop_rewriting(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(comment);
    (void)(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

EXPECT_OUTPUT(
    test_comment_api_output1,
    "<div><!--Yo-->&lt;/div&gt;"
);

EXPECT_OUTPUT(
    test_comment_api_output2,
    "<div><repl><after></div>"
);

EXPECT_OUTPUT(
    test_comment_api_output3,
    "<>"
);

void test_comment_api() {
    int user_data = 42;

    const char *selector_str = "*";

    cool_thing_selector_t *selector = cool_thing_selector_parse(
        selector_str,
        strlen(selector_str)
    );

    REWRITE(
        "<!--Hey 42-->",
        test_comment_api_output1,
        {
             cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                &test_comment_api_comment_handler1,
                &user_data,
                NULL,
                NULL
            );

            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                &test_comment_api_user_data_get,
                NULL,
                NULL,
                NULL
            );
        }
    );

    REWRITE(
        "<div><!--Hello--></div>",
        test_comment_api_output2,
        {
            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                NULL,
                NULL,
                &test_comment_api_comment_handler2_el,
                NULL,
                NULL,
                NULL
            );

            ok(!err);

            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                &test_comment_api_comment_handler2_doc,
                NULL,
                NULL,
                NULL
            );
        }
    );

    REWRITE(
        "<<!--0_0-->>",
        test_comment_api_output3,
        {
             cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                &test_comment_api_comment_handler3,
                NULL,
                NULL,
                NULL
            );
        }
    );

    EXPECT_STOP(
        "<!-- hey -->",
        {
             cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                &test_comment_api_stop_rewriting,
                NULL,
                NULL,
                NULL
            );
        }
    );

    EXPECT_STOP(
        "<div><!-- foo --></div>",
        {
            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                NULL,
                NULL,
                &test_comment_api_stop_rewriting,
                NULL,
                NULL,
                NULL
            );

            ok(!err);
        }
    );

    cool_thing_selector_free(selector);
}
