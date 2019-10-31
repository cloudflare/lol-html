#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static int EXPECTED_USER_DATA = 43;

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    modify_tag_name_output_sink,
    "Hi <span>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t modify_element_tag_name(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

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

    return COOL_THING_CONTINUE;
}

static void test_modify_element_tag_name(cool_thing_selector_t *selector, void *user_data) {
    UNUSED(user_data);

    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &modify_element_tag_name,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(builder, "Hi <div>", modify_tag_name_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    modify_user_data_output_sink,
    "Hi <span>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t modify_user_data(
    cool_thing_element_t *element,
    void *user_data
) {
    note("User data");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    note("Set element user data");
    cool_thing_element_user_data_set(element, user_data);

    note("Get element user data");
    int element_user_data = *(int*)cool_thing_element_user_data_get(element);

    ok(element_user_data == EXPECTED_USER_DATA);

    return COOL_THING_CONTINUE;
}

static void test_modify_element_user_data(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &modify_user_data,
        user_data,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(builder, "Hi <span>", modify_user_data_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_and_modify_attributes_output_sink,
    "<span bar=\"hey\">",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t get_and_modify_attributes(
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

static void test_get_and_modify_attributes(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &get_and_modify_attributes,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(builder, "<span foo=42>", get_and_modify_attributes_output_sink, user_data);
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    insert_content_around_element_output_sink,
    "&amp;before<div><!--prepend-->Hi<!--append--></div>&amp;after",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static cool_thing_rewriter_directive_t insert_content_around_element(
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

static void test_insert_content_around_element(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &insert_content_around_element,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(
        builder,
        "<div>Hi</div>",
        insert_content_around_element_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    set_element_inner_content_output_sink,
    "<div>hey &amp; ya</div>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

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

static void test_set_element_inner_content(cool_thing_selector_t *selector, void *user_data) {
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

    run_rewriter(
        builder,
        "<div><span>42</span></div>",
        set_element_inner_content_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    replace_element_output_sink,
    "hey & ya<h1>Hellohey & ya</h1><h2>Hello2</h2>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

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

static void test_replace_element(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &replace_element,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(
        builder,
        "<div><span>42</span></div><h1>Hello<div>good bye</div></h1><h2>Hello2</h2>",
        replace_element_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    remove_element_output_sink,
    "<div><span>42</span></div><h2>Hello2</h2>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

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

static void test_remove_element(cool_thing_selector_t *selector, void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &remove_element,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(
        builder,
        "<div><span>42</span></div><h1>Hello</h1><h2>Hello2</h2>",
        remove_element_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    remove_element_and_keep_content_output_sink,
    "<div><span>42Hello1</span></div><h1>Hello</h1>Hello2",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

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

static void test_remove_element_and_keep_content(
        cool_thing_selector_t *selector,
        void *user_data
) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    int err = cool_thing_rewriter_builder_add_element_content_handlers(
        builder,
        selector,
        &remove_element_and_keep_content,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    ok(!err);

    run_rewriter(
        builder,
        "<div><span>42<h2>Hello1</h2></span></div><h1>Hello</h1><h2>Hello2</h2>",
        remove_element_and_keep_content_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_and_free_empty_attribute_output_sink,
    "<span foo>",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

static void test_get_and_free_empty_element_attribute(
    cool_thing_selector_t *selector,
    void *user_data
) {
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

    run_rewriter(builder, "<span foo>", get_and_free_empty_attribute_output_sink, user_data);
}

//-------------------------------------------------------------------------
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

static void test_iterate_attributes(cool_thing_selector_t *selector, void *user_data) {
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

    run_rewriter(builder, "<div foo=42 bar='1337'>", output_sink_stub, user_data);
}

//-------------------------------------------------------------------------
static cool_thing_rewriter_directive_t assert_element_ns_is_html(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *ns = cool_thing_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/1999/xhtml");

    return COOL_THING_CONTINUE;
}

static void test_element_ns_is_html(cool_thing_selector_t *selector, void *user_data) {
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

    run_rewriter(builder, "<script></script>", output_sink_stub, user_data);
}

//-------------------------------------------------------------------------
static cool_thing_rewriter_directive_t assert_element_ns_is_svg(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *ns = cool_thing_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/2000/svg");

    return COOL_THING_CONTINUE;
}

static void test_element_ns_is_svg(cool_thing_selector_t *selector, void *user_data) {
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

    run_rewriter(builder, "<svg><script></script></svg>", output_sink_stub, user_data);
}

//-------------------------------------------------------------------------
static cool_thing_rewriter_directive_t stop_rewriting(
    cool_thing_element_t *element,
    void *user_data
) {
    UNUSED(element);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

static void test_stop(cool_thing_selector_t *selector, void *user_data) {
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

    expect_stop(builder, "<span foo>", user_data);

    ok(!err);
}

void element_api_test() {
    int user_data = 43;

    {
        const char *selector_str = "*";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_modify_element_tag_name(selector, &user_data);
        test_modify_element_user_data(selector, &user_data);
        test_iterate_attributes(selector, &user_data);
        test_get_and_modify_attributes(selector, &user_data);
        test_insert_content_around_element(selector, &user_data);

        cool_thing_selector_free(selector);
    }

    {
        const char *selector_str = "div";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_set_element_inner_content(selector, &user_data);

        cool_thing_selector_free(selector);
    }

    {
        const char *selector_str = "div";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_replace_element(selector, &user_data);

        cool_thing_selector_free(selector);
    }

    {
        const char *selector_str = "h1";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_remove_element(selector, &user_data);

        cool_thing_selector_free(selector);
    }

    {
        const char *selector_str = "h2";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_remove_element_and_keep_content(selector, &user_data);

        cool_thing_selector_free(selector);
    }

    {
        const char *selector_str = "span";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_get_and_free_empty_element_attribute(selector, &user_data);
        test_stop(selector, &user_data);

        cool_thing_selector_free(selector);
    }

    {
        note("NamespaceURI");

        const char *selector_str = "script";

        cool_thing_selector_t *selector = cool_thing_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_element_ns_is_html(selector, &user_data);
        test_element_ns_is_svg(selector, &user_data);
    }
}
