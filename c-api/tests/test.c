
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "deps/picotest/picotest.h"
#include "../include/cool_thing.h"

#define EXPECT_OUTPUT(sink_name, expected) \
    static void sink_name(const char *chunk, size_t chunk_len) { \
        static char *out = NULL; \
        static size_t out_len = 0; \
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


#define REWRITE(html, output_sink, assign_handlers) \
    do { \
        const char *in = html; \
        const char *encoding = "UTF-8"; \
        cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new(); \
    \
        assign_handlers \
    \
        cool_thing_rewriter_t *rewriter = cool_thing_rewriter_build( \
            builder, \
            encoding, \
            strlen(encoding), \
            &output_sink \
        ); \
    \
        ok(!cool_thing_rewriter_write(rewriter, in, strlen(in))); \
        ok(!cool_thing_rewriter_end(rewriter)); \
        cool_thing_rewriter_free(rewriter); \
    } while(0)


#define str_eq(actual, expected) { \
    ok((actual) != NULL); \
    ok((actual)->len == strlen(expected)); \
    ok(!memcmp((actual)->data, expected, (actual)->len)); \
}

static void output_sink_stub(const char *chunk, size_t chunk_len) {
    (void)(chunk);
    (void)(chunk_len);
}


// Unsupported selector
//---------------------------------------------------------------------
static void test_unsupported_selector() {
    const char *selector = "p:last-child";
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        strlen(selector),
        NULL,
        NULL,
        NULL
    );

    ok(err == -1);

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
        &output_sink_stub
    );

    ok(rewriter == NULL);

    cool_thing_str_t *msg = cool_thing_take_last_error();

    str_eq(msg, "Expected ASCII-compatible encoding.");

    cool_thing_str_free(*msg);
}

// Doctype API
//---------------------------------------------------------------------
static void test_doctype_api_doctype_handler(cool_thing_doctype_t *doctype) {
    cool_thing_str_t *name = cool_thing_doctype_name_get(doctype);
    cool_thing_str_t *public_id = cool_thing_doctype_public_id_get(doctype);
    cool_thing_str_t *system_id = cool_thing_doctype_system_id_get(doctype);

    str_eq(name, "math");
    ok(public_id == NULL);
    str_eq(system_id, "http://www.w3.org/Math/DTD/mathml1/mathml.dtd");

    cool_thing_str_free(*name);
    cool_thing_str_free(*system_id);
}

EXPECT_OUTPUT(
    test_doctype_api_output,
    "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">"
)

static void test_doctype_api() {
    REWRITE(
        "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
        test_doctype_api_output,
        {
            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                &test_doctype_api_doctype_handler,
                NULL,
                NULL
            );
        }
    );
}

// Comment API
//---------------------------------------------------------------------
static void test_comment_api_comment_handler1(cool_thing_comment_t *comment) {
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
}

static void test_comment_api_comment_handler2_el(cool_thing_comment_t *comment) {
    const char *replacement = "<repl>";

    note("Replace");
    ok(!cool_thing_comment_replace(comment, replacement, strlen(replacement), true));
    ok(cool_thing_comment_is_removed(comment));
}

static void test_comment_api_comment_handler2_doc(cool_thing_comment_t *comment) {
    const char *after = "<after>";

    note("Insert after replaced");
    ok(!cool_thing_comment_after(comment, after, strlen(after), true));
}

static void test_comment_api_comment_handler3(cool_thing_comment_t *comment) {
    note("Remove");
    cool_thing_comment_remove(comment);
    ok(cool_thing_comment_is_removed(comment));
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
    REWRITE(
        "<!--Hey 42-->",
        test_comment_api_output1,
        {
             cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                &test_comment_api_comment_handler1,
                NULL
            );
        }
    );

    REWRITE(
        "<div><!--Hello--></div>",
        test_comment_api_output2,
        {
            const char *selector = "*";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                strlen(selector),
                NULL,
                &test_comment_api_comment_handler2_el,
                NULL
            );

            ok(!err);

            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                &test_comment_api_comment_handler2_doc,
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
                &test_comment_api_comment_handler3,
                NULL
            );
        }
    );
}

// Text chunk API
//---------------------------------------------------------------------
static void test_text_chunk_api_text_chunk_handler1(cool_thing_text_chunk_t *chunk) {
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
}

static void test_text_chunk_api_text_chunk_handler2_el(cool_thing_text_chunk_t *chunk) {
    const char *replacement = "<repl>";

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Replace");
        ok(!cool_thing_text_chunk_replace(chunk, replacement, strlen(replacement), true));
        ok(cool_thing_text_chunk_is_removed(chunk));
    }
}

static void test_text_chunk_api_text_chunk_handler2_doc(cool_thing_text_chunk_t *chunk) {
    const char *after = "<after>";

    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Insert after replaced");
        ok(!cool_thing_text_chunk_after(chunk, after, strlen(after), true));
    }
}

static void test_text_chunk_api_text_chunk_handler3(cool_thing_text_chunk_t *chunk) {
    if (cool_thing_text_chunk_content_get(chunk).len > 0) {
        note("Remove");
        cool_thing_text_chunk_remove(chunk);
        ok(cool_thing_text_chunk_is_removed(chunk));
    }
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

static void test_text_chunk_api() {
    REWRITE(
        "Hey 42",
        test_text_chunk_api_output1,
        {
             cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                &test_text_chunk_api_text_chunk_handler1
            );
        }
    );

    REWRITE(
        "<div>Hello</div>",
        test_text_chunk_api_output2,
        {
            const char *selector = "*";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                strlen(selector),
                NULL,
                NULL,
                &test_text_chunk_api_text_chunk_handler2_el
            );

            ok(!err);

            cool_thing_rewriter_builder_add_document_content_handlers(
                builder,
                NULL,
                NULL,
                &test_text_chunk_api_text_chunk_handler2_doc
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
                &test_text_chunk_api_text_chunk_handler3
            );
        }
    );
}

// Element
//---------------------------------------------------------------------
static void modify_element_tag_name(cool_thing_element_t *element) {
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
}

static void iterate_element_attributes(cool_thing_element_t *element) {
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
}

static void get_and_modify_element_attributes(cool_thing_element_t *element) {
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
}

static void element_surrounding_content_insertion(cool_thing_element_t *element) {
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
}

static void set_element_inner_content(cool_thing_element_t *element) {
    const char *content = "hey & ya";

    note("Set inner content");
    ok(!cool_thing_element_set_inner_content(element, content, strlen(content), false));
}

static void replace_element(cool_thing_element_t *element) {
    const char *content = "hey & ya";

    note("Replace");
    ok(!cool_thing_element_replace(element, content, strlen(content), true));
}

static void remove_element(cool_thing_element_t *element) {
    note("Remove");
    ok(!cool_thing_element_is_removed(element));
    cool_thing_element_remove(element);
    ok(cool_thing_element_is_removed(element));
}

static void remove_element_and_keep_content(cool_thing_element_t *element) {
    note("Remove and keep content");
    ok(!cool_thing_element_is_removed(element));
    cool_thing_element_remove_and_keep_content(element);
    ok(cool_thing_element_is_removed(element));
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
    REWRITE(
        "Hi <div>",
        test_element_api_output1,
        {
            const char *selector = "*";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                strlen(selector),
                &modify_element_tag_name,
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
            const char *selector = "*";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                strlen(selector),
                &iterate_element_attributes,
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
            const char *selector = "*";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                strlen(selector),
                &get_and_modify_element_attributes,
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
            const char *selector = "*";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                strlen(selector),
                &element_surrounding_content_insertion,
                NULL,
                NULL
            );

            ok(!err);
        }
    );


    REWRITE(
        "<div><span>42</span></div>",
        test_element_api_output4,
        {
            const char *selector = "div";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector,
                strlen(selector),
                &set_element_inner_content,
                NULL,
                NULL
            );

            ok(!err);
        }
    );

    REWRITE(
        "<div><span>42</span></div><h1>Hello</h1><h2>Hello2</h2>",
        test_element_api_output5,
        {
            const char *selector1 = "div";
            const char *selector2 = "h1";
            const char *selector3 = "h2";

            int err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector1,
                strlen(selector1),
                &replace_element,
                NULL,
                NULL
            );

            ok(!err);

            err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector2,
                strlen(selector2),
                &remove_element,
                NULL,
                NULL
            );

            ok(!err);

            err = cool_thing_rewriter_builder_add_element_content_handlers(
                builder,
                selector3,
                strlen(selector3),
                &remove_element_and_keep_content,
                NULL,
                NULL
            );

            ok(!err);
        }
    );
}

int main() {
    subtest("Unsupported selector", test_unsupported_selector);
    subtest("Non-ASCII encoding", test_non_ascii_encoding);
    subtest("Doctype API", test_doctype_api);
    subtest("Comment API", test_comment_api);
    subtest("Text chunk API", test_text_chunk_api);
    subtest("Element API", element_api_test);

    return done_testing();
}