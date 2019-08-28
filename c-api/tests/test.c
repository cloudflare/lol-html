
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include "deps/picotest/picotest.h"
#include "../include/cool_thing.h"

#define EXPECT_OUTPUT(sink_name, expected) \
    static void sink_name(const char *chunk, size_t chunk_len, void *user_data) { \
        static char *out = NULL; \
        static size_t out_len = 0; \
    \
        ok(*(int*)user_data == 42); \
    \
        if (chunk_len > 0) { \
            out = (char *) (out == NULL ? malloc(chunk_len) : realloc(out, out_len + chunk_len)); \
            memcpy(out + out_len, chunk, chunk_len); \
            out_len += chunk_len; \
        } else { \
            ok(out_len == strlen(expected)); \
            ok(!memcmp(out, expected, out_len)); \
        } \
    }

#define RUN_REWRITER_WITH_MAX_MEMORY(html, output_sink, assign_handlers, actions, max_memory) \
    do { \
        const char *in = html; \
        const char *encoding = "UTF-8"; \
        int output_sink_user_data = 42; \
        cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new(); \
    \
        assign_handlers \
    \
        cool_thing_rewriter_t *rewriter = cool_thing_rewriter_build( \
            builder, \
            encoding, \
            strlen(encoding), \
            0, /* initial_memory */ \
            max_memory, \
            &output_sink, \
            &output_sink_user_data, \
            true \
        ); \
    \
        cool_thing_rewriter_builder_free(builder); \
        actions \
        cool_thing_rewriter_free(rewriter); \
    } while(0)

#define RUN_REWRITER(html, output_sink, assign_handlers, actions) \
    RUN_REWRITER_WITH_MAX_MEMORY(html, output_sink, assign_handlers, actions, 2048)

#define REWRITE(html, output_sink, assign_handlers) \
    RUN_REWRITER(html, output_sink, assign_handlers, { \
        ok(!cool_thing_rewriter_write(rewriter, in, strlen(in))); \
        ok(!cool_thing_rewriter_end(rewriter)); \
    })

#define EXPECT_STOP(html, assign_handlers) \
    RUN_REWRITER(html, output_sink_stub, assign_handlers, { \
        ok(cool_thing_rewriter_write(rewriter, in, strlen(in))); \
    \
        cool_thing_str_t *msg = cool_thing_take_last_error(); \
    \
        str_eq(msg, "The rewriter has been stopped."); \
    \
        cool_thing_str_free(*msg); \
    })

#define str_eq(actual, expected) { \
    ok((actual) != NULL); \
    ok((actual)->len == strlen(expected)); \
    ok(!memcmp((actual)->data, expected, (actual)->len)); \
}

#define str_contains(actual, expected) { \
    ok((actual) != NULL); \
    ok(strstr(actual->data, expected) != NULL); \
}

#define c_str_eq(actual, expected) ok(!strcmp(actual, expected))

static void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data) {
    (void)(chunk);
    (void)(chunk_len);
    (void)(user_data);
}


// Unsupported selector
//---------------------------------------------------------------------
static void test_unsupported_selector() {
    const char *selector_str = "p:last-child";
    cool_thing_selector_t *selector = cool_thing_selector_parse(selector_str, strlen(selector_str));

    ok(selector == NULL);

    cool_thing_str_t *msg = cool_thing_take_last_error();

    str_eq(msg, "Unsupported pseudo-class or pseudo-element in selector.");

    cool_thing_str_free(*msg);
}

// Non-ASCII encoding
//---------------------------------------------------------------------
static void test_non_ascii_encoding() {
    const char *encoding = "UTF-16";
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_t *rewriter = cool_thing_rewriter_build(
        builder,
        encoding,
        strlen(encoding),
        0, // initial_memory
        16, // max_memory
        &output_sink_stub,
        NULL,
        true
    );

    cool_thing_rewriter_builder_free(builder);

    ok(rewriter == NULL);

    cool_thing_str_t *msg = cool_thing_take_last_error();

    str_eq(msg, "Expected ASCII-compatible encoding.");

    cool_thing_str_free(*msg);
}

// Doctype API
//---------------------------------------------------------------------
static cool_thing_rewriter_directive_t test_doctype_api_doctype_handler(
    cool_thing_doctype_t *doctype,
    void *user_data
) {
    note("Fields");

    cool_thing_str_t *name = cool_thing_doctype_name_get(doctype);
    cool_thing_str_t *public_id = cool_thing_doctype_public_id_get(doctype);
    cool_thing_str_t *system_id = cool_thing_doctype_system_id_get(doctype);

    str_eq(name, "math");
    ok(public_id == NULL);
    str_eq(system_id, "http://www.w3.org/Math/DTD/mathml1/mathml.dtd");

    cool_thing_str_free(*name);
    cool_thing_str_free(*system_id);

    note("User data");
    ok(*(int*)user_data == 42);

    note("Set doctype user data");
    cool_thing_doctype_user_data_set(doctype, user_data);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_doctype_api_user_data_get(
    cool_thing_doctype_t *doctype,
    void *user_data
) {
    (void)(user_data);

    note("Get doctype user data");

    int doctype_user_data = *(int*)cool_thing_doctype_user_data_get(doctype);

    ok(doctype_user_data == 42);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_doctype_api_stop_rewriting (
    cool_thing_doctype_t *doctype,
    void *user_data
) {
    (void)(doctype);
    (void)(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

EXPECT_OUTPUT(
    test_doctype_api_output,
    "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">"
)

static void test_doctype_api() {
    int user_data = 42;

    REWRITE(
        "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
        test_doctype_api_output,
        {
            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                &test_doctype_api_doctype_handler,
                &user_data,
                NULL,
                NULL,
                NULL,
                NULL
            );

            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                &test_doctype_api_user_data_get,
                NULL,
                NULL,
                NULL,
                NULL,
                NULL
            );
        }
    );

    EXPECT_STOP(
        "<!doctype>",
        {
            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                &test_doctype_api_stop_rewriting ,
                NULL,
                NULL,
                NULL,
                NULL,
                NULL
            );
        }
    );
}

// Comment API
//---------------------------------------------------------------------
static cool_thing_rewriter_directive_t test_comment_api_comment_handler1(
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

static cool_thing_rewriter_directive_t test_comment_api_user_data_get(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(user_data);

    note("Get comment user data");

    int comment_user_data = *(int*)cool_thing_comment_user_data_get(comment);

    ok(comment_user_data == 42);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_comment_api_comment_handler2_el(
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

static cool_thing_rewriter_directive_t test_comment_api_comment_handler2_doc(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(user_data);

    const char *after = "<after>";

    note("Insert after replaced");
    ok(!cool_thing_comment_after(comment, after, strlen(after), true));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_comment_api_comment_handler3(
    cool_thing_comment_t *comment,
    void *user_data
) {
    (void)(user_data);

    note("Remove");
    cool_thing_comment_remove(comment);
    ok(cool_thing_comment_is_removed(comment));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_comment_api_stop_rewriting(
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

static void test_comment_api() {
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

// Text chunk API
//---------------------------------------------------------------------
static cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler1(
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

static cool_thing_rewriter_directive_t test_text_chunk_api_user_data_get(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    (void)(user_data);

    note("Get text chunk user data");

    int chunk_user_data = *(int*)cool_thing_text_chunk_user_data_get(chunk);

    ok(chunk_user_data == 42);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler2_el(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    (void)(user_data);
    const char *replacement = "<repl>";

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Replace");
        ok(!cool_thing_text_chunk_replace(chunk, replacement, strlen(replacement), true));
        ok(cool_thing_text_chunk_is_removed(chunk));
    }

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler2_doc(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    (void)(user_data);
    const char *after = "<after>";

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Insert after replaced");
        ok(!cool_thing_text_chunk_after(chunk, after, strlen(after), true));
    }

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_text_chunk_api_text_chunk_handler3(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    (void)(user_data);

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Remove");
        cool_thing_text_chunk_remove(chunk);
        ok(cool_thing_text_chunk_is_removed(chunk));
    }

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_text_chunk_api_stop_rewriting(
    cool_thing_text_chunk_t *chunk,
    void *user_data
) {
    (void)(chunk);
    (void)(user_data);

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

EXPECT_OUTPUT(
    test_element_api_output6,
    "<span foo>"
);

static void test_text_chunk_api() {
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

// Element
//---------------------------------------------------------------------
static cool_thing_rewriter_directive_t modify_element_tag_name(
    cool_thing_element_t *element,
    void *user_data
) {
    const char *new_name = "span";

    note("Get tag name");
    cool_thing_str_t name = cool_thing_element_tag_name_get(element);

    str_eq(&name, "div");

    cool_thing_str_free(name);

    note("Set invalid tag name");
    ok(cool_thing_element_tag_name_set(element, "", 0) == -1);

    cool_thing_str_t *msg = cool_thing_take_last_error();

    str_eq(msg, "Tag name can't be empty.");

    cool_thing_str_free(*msg);

    note("Set tag name");
    ok(!cool_thing_element_tag_name_set(element, new_name, strlen(new_name)));

    note("User data");
    ok(*(int*)user_data == 42);

    note("Set element user data");
    cool_thing_element_user_data_set(element, user_data);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_element_api_user_data_get(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    note("Get element user data");

    int element_user_data = *(int*)cool_thing_element_user_data_get(element);

    ok(element_user_data == 42);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t iterate_element_attributes(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    note("Attributes iterator");
    cool_thing_attributes_iterator_t *iter = cool_thing_attributes_iterator_get(element);

    const cool_thing_attribute_t *attr = cool_thing_attributes_iterator_next(iter);

    ok(attr != NULL);

    cool_thing_str_t name = cool_thing_attribute_name_get(attr);
    cool_thing_str_t value = cool_thing_attribute_value_get(attr);

    str_eq(&name, "foo");
    str_eq(&value, "42");

    cool_thing_str_free(name);
    cool_thing_str_free(value);

    attr = cool_thing_attributes_iterator_next(iter);

    ok(attr != NULL);

    name = cool_thing_attribute_name_get(attr);
    value = cool_thing_attribute_value_get(attr);

    str_eq(&name, "bar");
    str_eq(&value, "1337");

    cool_thing_str_free(name);
    cool_thing_str_free(value);

    attr = cool_thing_attributes_iterator_next(iter);

    ok(attr == NULL);

    cool_thing_attributes_iterator_free(iter);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t get_and_modify_element_attributes(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    const char *attr1 = "foo";
    const char *attr2 = "Bar";
    const char *attr2_value = "hey";

    note("Has attribute");
    ok(cool_thing_element_has_attribute(element, attr1, strlen(attr1)) == 1);
    ok(!cool_thing_element_has_attribute(element, attr2, strlen(attr2)));

    note("Get attribute");
    cool_thing_str_t *value = cool_thing_element_get_attribute(
        element,
        attr1,
        strlen(attr1)
    );

    str_eq(value, "42");

    value = cool_thing_element_get_attribute(
        element,
        attr2,
        strlen(attr2)
    );

    ok(value == NULL);

    note("Set attribute");
    int err = cool_thing_element_set_attribute(
        element,
        attr2,
        strlen(attr2),
        attr2_value,
        strlen(attr2_value)
    );

    ok(!err);

    note("Remove attribute");
    ok(!cool_thing_element_remove_attribute(element, attr1, strlen(attr1)));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t element_surrounding_content_insertion(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    const char *before = "&before";
    const char *prepend = "<!--prepend-->";
    const char *append = "<!--append-->";
    const char *after = "&after";

    note("Insert before/prepend");
    ok(!cool_thing_element_before(element, before, strlen(before), false));
    ok(!cool_thing_element_prepend(element, prepend, strlen(prepend), true));

    note("Insert after/append");
    ok(!cool_thing_element_append(element, append, strlen(append), true));
    ok(!cool_thing_element_after(element, after, strlen(after), false));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t set_element_inner_content(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    const char *content = "hey & ya";

    note("Set inner content");
    ok(!cool_thing_element_set_inner_content(element, content, strlen(content), false));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t replace_element(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    const char *content = "hey & ya";

    note("Replace");
    ok(!cool_thing_element_replace(element, content, strlen(content), true));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t remove_element(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    note("Remove");
    ok(!cool_thing_element_is_removed(element));
    cool_thing_element_remove(element);
    ok(cool_thing_element_is_removed(element));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t remove_element_and_keep_content(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    note("Remove and keep content");
    ok(!cool_thing_element_is_removed(element));
    cool_thing_element_remove_and_keep_content(element);
    ok(cool_thing_element_is_removed(element));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t get_and_free_empty_element_attribute(
    cool_thing_element_t *element, \
    void *user_data
) {
    (void)(user_data);

    const char *attr1 = "foo";

    note("Has attribute");
    ok(cool_thing_element_has_attribute(element, attr1, strlen(attr1)) == 1);

    note("Get attribute");
    cool_thing_str_t *value = cool_thing_element_get_attribute(
        element,
        attr1,
        strlen(attr1)
    );

    str_eq(value, "");
    cool_thing_str_free(*value);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t test_element_api_stop_rewriting(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(element);
    (void)(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

static cool_thing_rewriter_directive_t assert_element_ns_is_html(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    const char *ns = cool_thing_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/1999/xhtml");

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t assert_element_ns_is_svg(
    cool_thing_element_t *element,
    void *user_data
) {
    (void)(user_data);

    const char *ns = cool_thing_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/2000/svg");

    return COOL_THING_CONTINUE;
}

EXPECT_OUTPUT(
    test_element_api_output1,
    "Hi <span>"
);

EXPECT_OUTPUT(
    test_element_api_output2,
    "<span bar=\"hey\">"
);

EXPECT_OUTPUT(
    test_element_api_output3,
    "&amp;before<div><!--prepend-->Hi<!--append--></div>&amp;after"
);

EXPECT_OUTPUT(
    test_element_api_output4,
    "<div>hey &amp; ya</div>"
);

EXPECT_OUTPUT(
    test_element_api_output5,
    "hey & yaHello2"
);

static void element_api_test() {
    int user_data = 42;

    {
        const char *selector_str = "*";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        REWRITE(
            "Hi <div>",
            test_element_api_output1,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &modify_element_tag_name,
                    &user_data,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);

                err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &test_element_api_user_data_get,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        REWRITE(
            "<div foo=42 bar='1337'>",
            output_sink_stub,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &iterate_element_attributes,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        REWRITE(
            "<span foo=42>",
            test_element_api_output2,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &get_and_modify_element_attributes,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        REWRITE(
            "<div>Hi</div>",
            test_element_api_output3,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &element_surrounding_content_insertion,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        cool_thing_selector_free(selector);
    }

    {
        const char *selector_str = "div";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        REWRITE(
            "<div><span>42</span></div>",
            test_element_api_output4,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &set_element_inner_content,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        cool_thing_selector_free(selector);
    }

    {
        const char *selector1_str = "div";

        cool_thing_selector_t *selector1 = cool_thing_selector_parse(
            selector1_str,
            strlen(selector1_str)
        );

        const char *selector2_str = "h1";

        cool_thing_selector_t *selector2 = cool_thing_selector_parse(
            selector2_str,
            strlen(selector2_str)
        );

        const char *selector3_str = "h2";

        cool_thing_selector_t *selector3 = cool_thing_selector_parse(
            selector3_str,
            strlen(selector3_str)
        );

        REWRITE(
            "<div><span>42</span></div><h1>Hello</h1><h2>Hello2</h2>",
            test_element_api_output5,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector1,
                    &replace_element,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);

                err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector2,
                    &remove_element,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);

                err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector3,
                    &remove_element_and_keep_content,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        cool_thing_selector_free(selector1);
        cool_thing_selector_free(selector2);
        cool_thing_selector_free(selector3);
    }

    {
        const char *selector_str = "span";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        REWRITE(
            "<span foo>",
            test_element_api_output6,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &get_and_free_empty_element_attribute,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        EXPECT_STOP(
            "<span foo>",
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &test_element_api_stop_rewriting,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        cool_thing_selector_free(selector);
    }

    {

        note("NamespaceURI");

        const char *selector_str = "script";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        REWRITE(
            "<script></script>",
            output_sink_stub,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &assert_element_ns_is_html,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );

        REWRITE(
            "<svg><script></script></svg>",
            output_sink_stub,
            {
                int err = cool_thing_rewriter_builder_add_element_content_handlers(
                    builder,
                    selector,
                    &assert_element_ns_is_svg,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    NULL
                );

                ok(!err);
            }
        );
    }

}

// Out of memory
//---------------------------------------------------------------------

static void test_out_of_memory() {
    const char *chunk1 = "<span alt='aaaaa";

    RUN_REWRITER_WITH_MAX_MEMORY(chunk1, output_sink_stub,
        {
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
        },
        {
            ok(cool_thing_rewriter_write(rewriter, in, strlen(in)) == -1);

            cool_thing_str_t *msg = cool_thing_take_last_error();
            ok(msg != NULL);

            str_contains(msg, "exceeded limits.");
            cool_thing_str_free(*msg);
        },
    5);
}

int main() {
    subtest("Unsupported selector", test_unsupported_selector);
    subtest("Non-ASCII encoding", test_non_ascii_encoding);
    subtest("Doctype API", test_doctype_api);
    subtest("Comment API", test_comment_api);
    subtest("Text chunk API", test_text_chunk_api);
    subtest("Element API", element_api_test);
    subtest("Out of memory", test_out_of_memory);

    return done_testing();
}
