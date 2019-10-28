#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

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

static cool_thing_rewriter_directive_t user_data_get(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    note("Get element user data");

    int element_user_data = *(int*)cool_thing_element_user_data_get(element);

    ok(element_user_data == 42);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t iterate_element_attributes(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

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
    UNUSED(user_data);

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
    UNUSED(user_data);

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
    UNUSED(user_data);

    const char *content = "hey & ya";

    note("Set inner content");
    ok(!cool_thing_element_set_inner_content(element, content, strlen(content), false));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t replace_element(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *content = "hey & ya";

    note("Replace");
    ok(!cool_thing_element_replace(element, content, strlen(content), true));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t remove_element(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

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
    UNUSED(user_data);

    note("Remove and keep content");
    ok(!cool_thing_element_is_removed(element));
    cool_thing_element_remove_and_keep_content(element);
    ok(cool_thing_element_is_removed(element));

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t stop_rewriting(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(element);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

static cool_thing_rewriter_directive_t assert_element_ns_is_html(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *ns = cool_thing_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/1999/xhtml");

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t assert_element_ns_is_svg(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *ns = cool_thing_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/2000/svg");

    return COOL_THING_CONTINUE;
}

EXPECT_OUTPUT(
    test_element_api_output1,
    "Hi <span>"
);

void test_output1(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &modify_element_tag_name,
        user_data,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &user_data_get,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(builder, "Hi <div>", test_element_api_output1);
}

EXPECT_OUTPUT(
    test_element_api_output2,
    "<span bar=\"hey\">"
);

void test_output2(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder, "<span foo=42>", test_element_api_output2);
}

EXPECT_OUTPUT(
    test_element_api_output3,
    "&amp;before<div><!--prepend-->Hi<!--append--></div>&amp;after"
);

void test_output3(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder, "<div>Hi</div>", test_element_api_output3);
}

EXPECT_OUTPUT(
    test_element_api_output4,
    "<div>hey &amp; ya</div>"
);

void test_output4(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder, "<div><span>42</span></div>", test_element_api_output4);
}

EXPECT_OUTPUT(
    test_element_api_output5,
    "hey & yaHello2"
);

void test_output5(cool_thing_selector_t *selector1,
    cool_thing_selector_t *selector2,
    cool_thing_selector_t *selector3
) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder,
        "<div><span>42</span></div><h1>Hello</h1><h2>Hello2</h2>",
        test_element_api_output5
    );
}

EXPECT_OUTPUT(
    test_element_api_output6,
    "<span foo>"
);

void test_output6(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder, "<span foo>", test_element_api_output6);
}

void test_output_sink_stub1(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder, "<div foo=42 bar='1337'>", output_sink_stub);
}

void test_output_sink_stub2(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder, "<script></script>", output_sink_stub);
}

void test_output_sink_stub3(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

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

    run_rewriter(builder, "<svg><script></script></svg>", output_sink_stub);
}

void test_expect_stop(cool_thing_selector_t *selector) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &stop_rewriting,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    expect_stop(builder, "<span foo>");

    ok(!err);
}

void element_api_test() {
    int user_data = 42;

    {
        const char *selector_str = "*";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_output1(selector, &user_data);
        test_output_sink_stub1(selector);
        test_output2(selector);
        test_output3(selector);

        cool_thing_selector_free(selector);
    }

    {
        const char *selector_str = "div";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_output4(selector);

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

        test_output5(selector1, selector2, selector3);

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

        test_output6(selector);
        test_expect_stop(selector);

        cool_thing_selector_free(selector);
    }

    {
        note("NamespaceURI");

        const char *selector_str = "script";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_output_sink_stub2(selector);
        test_output_sink_stub3(selector);
    }
}
