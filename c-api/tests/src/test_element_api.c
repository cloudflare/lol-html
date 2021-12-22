#include "../../include/lol_html.h"
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

static lol_html_rewriter_directive_t modify_element_tag_name(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *new_name = "span";

    note("Get tag name");
    lol_html_str_t name = lol_html_element_tag_name_get(element);

    str_eq(name, "div");

    lol_html_str_free(name);

    note("Set invalid tag name");
    ok(lol_html_element_tag_name_set(element, "", 0) == -1);

    lol_html_str_t msg = lol_html_take_last_error();

    str_eq(msg, "Tag name can't be empty.");

    lol_html_str_free(msg);

    note("Set tag name");
    ok(!lol_html_element_tag_name_set(element, new_name, strlen(new_name)));

    return LOL_HTML_CONTINUE;
}

static void test_modify_element_tag_name(lol_html_selector_t *selector, void *user_data) {
    UNUSED(user_data);

    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t modify_user_data(
    lol_html_element_t *element,
    void *user_data
) {
    note("User data");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    note("Set element user data");
    lol_html_element_user_data_set(element, user_data);

    note("Get element user data");
    int element_user_data = *(int*)lol_html_element_user_data_get(element);

    ok(element_user_data == EXPECTED_USER_DATA);

    return LOL_HTML_CONTINUE;
}

static void test_modify_element_user_data(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t get_and_modify_attributes(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *attr1 = "foo";
    const char *attr2 = "Bar";
    const char *attr2_value = "hey";

    note("Has attribute");
    ok(lol_html_element_has_attribute(element, attr1, strlen(attr1)) == 1);
    ok(!lol_html_element_has_attribute(element, attr2, strlen(attr2)));

    note("Get attribute");
    lol_html_str_t value = lol_html_element_get_attribute(
        element,
        attr1,
        strlen(attr1)
    );

    str_eq(value, "42");

    value = lol_html_element_get_attribute(
        element,
        attr2,
        strlen(attr2)
    );

    ok(value.data == NULL);

    note("Set attribute");
    int err = lol_html_element_set_attribute(
        element,
        attr2,
        strlen(attr2),
        attr2_value,
        strlen(attr2_value)
    );

    ok(!err);

    note("Remove attribute");
    ok(!lol_html_element_remove_attribute(element, attr1, strlen(attr1)));

    return LOL_HTML_CONTINUE;
}

static void test_get_and_modify_attributes(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t insert_content_around_element(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *before = "&before";
    const char *prepend = "<!--prepend-->";
    const char *append = "<!--append-->";
    const char *after = "&after";

    note("Insert before/prepend");
    ok(!lol_html_element_before(element, before, strlen(before), false));
    ok(!lol_html_element_prepend(element, prepend, strlen(prepend), true));

    note("Insert after/append");
    ok(!lol_html_element_append(element, append, strlen(append), true));
    ok(!lol_html_element_after(element, after, strlen(after), false));

    return LOL_HTML_CONTINUE;
}

static void test_insert_content_around_element(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t set_element_inner_content(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *content = "hey & ya";

    note("Set inner content");
    ok(!lol_html_element_set_inner_content(element, content, strlen(content), false));

    return LOL_HTML_CONTINUE;
}

static void test_set_element_inner_content(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t replace_element(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *content = "hey & ya";

    note("Replace");
    ok(!lol_html_element_replace(element, content, strlen(content), true));

    return LOL_HTML_CONTINUE;
}

static void test_replace_element(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t remove_element(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    note("Remove");
    ok(!lol_html_element_is_removed(element));
    lol_html_element_remove(element);
    ok(lol_html_element_is_removed(element));

    return LOL_HTML_CONTINUE;
}

static void test_remove_element(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t remove_element_and_keep_content(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    note("Remove and keep content");
    ok(!lol_html_element_is_removed(element));
    lol_html_element_remove_and_keep_content(element);
    ok(lol_html_element_is_removed(element));

    return LOL_HTML_CONTINUE;
}

static void test_remove_element_and_keep_content(
        lol_html_selector_t *selector,
        void *user_data
) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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
    lol_html_selector_t *selector,
    void *user_data
) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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
static lol_html_rewriter_directive_t iterate_element_attributes(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    note("Attributes iterator");
    lol_html_attributes_iterator_t *iter = lol_html_attributes_iterator_get(element);

    const lol_html_attribute_t *attr = lol_html_attributes_iterator_next(iter);

    ok(attr != NULL);

    lol_html_str_t name = lol_html_attribute_name_get(attr);
    lol_html_str_t value = lol_html_attribute_value_get(attr);

    str_eq(name, "foo");
    str_eq(value, "42");

    lol_html_str_free(name);
    lol_html_str_free(value);

    attr = lol_html_attributes_iterator_next(iter);

    ok(attr != NULL);

    name = lol_html_attribute_name_get(attr);
    value = lol_html_attribute_value_get(attr);

    str_eq(name, "bar");
    str_eq(value, "1337");

    lol_html_str_free(name);
    lol_html_str_free(value);

    attr = lol_html_attributes_iterator_next(iter);

    ok(attr == NULL);

    lol_html_attributes_iterator_free(iter);

    return LOL_HTML_CONTINUE;
}

static void test_iterate_attributes(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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
static lol_html_rewriter_directive_t assert_element_ns_is_html(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *ns = lol_html_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/1999/xhtml");

    return LOL_HTML_CONTINUE;
}

static void test_element_ns_is_html(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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
static lol_html_rewriter_directive_t assert_element_ns_is_svg(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    const char *ns = lol_html_element_namespace_uri_get(element);

    c_str_eq(ns, "http://www.w3.org/2000/svg");

    return LOL_HTML_CONTINUE;
}

static void test_element_ns_is_svg(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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
static lol_html_rewriter_directive_t stop_rewriting(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(element);
    UNUSED(user_data);

    note("Stop rewriting");

    return LOL_HTML_STOP;
}

static void test_stop(lol_html_selector_t *selector, void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    int err = lol_html_rewriter_builder_add_element_content_handlers(
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

static lol_html_rewriter_directive_t modify_element_end_tag_name_inner(lol_html_end_tag_t *end_tag, void *user_data) {
    int times_run = *(int*)user_data;

    if (times_run == 0) {
        lol_ok(lol_html_end_tag_before(end_tag, "!", 1, false));
        const char *after_html = "<span>extra data</span>";
        lol_ok(lol_html_end_tag_after(end_tag, after_html, strlen(after_html), true));

        lol_html_str_t name = lol_html_end_tag_name_get(end_tag);
        str_eq(name, "div");

        lol_ok(lol_html_end_tag_name_set(end_tag, "div1", strlen("div1")));
        name = lol_html_end_tag_name_get(end_tag);
        str_eq(name, "div1");
    } else {
        lol_html_end_tag_remove(end_tag);
    }

    return LOL_HTML_CONTINUE;
}

static lol_html_rewriter_directive_t modify_element_end_tag_name_outer(
    lol_html_element_t *element,
    void *user_data
) {
    UNUSED(user_data);

    static int times_run = -1; // so that it will be 0 on the first call to `inner`

    lol_ok(lol_html_element_on_end_tag(element, modify_element_end_tag_name_inner, &times_run));
    times_run += 1;

    return LOL_HTML_CONTINUE;
}

EXPECT_OUTPUT(
    modify_element_end_tag,
    "<div>42!</div1><span>extra data</span><div>some data",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
);

void element_api_test() {
    int user_data = 43;

    {
        const char *selector_str = "*";

        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_modify_element_tag_name(selector, &user_data);
        test_modify_element_user_data(selector, &user_data);
        test_iterate_attributes(selector, &user_data);
        test_get_and_modify_attributes(selector, &user_data);
        test_insert_content_around_element(selector, &user_data);

        lol_html_selector_free(selector);
    }

    {
        const char *selector_str = "div";

        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_set_element_inner_content(selector, &user_data);

        lol_html_selector_free(selector);
    }

    {
        const char *selector_str = "div";

        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_replace_element(selector, &user_data);

        lol_html_selector_free(selector);
    }

    {
        const char *selector_str = "h1";

        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_remove_element(selector, &user_data);

        lol_html_selector_free(selector);
    }

    {
        const char *selector_str = "h2";

        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_remove_element_and_keep_content(selector, &user_data);

        lol_html_selector_free(selector);
    }

    {
        const char *selector_str = "span";

        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_get_and_free_empty_element_attribute(selector, &user_data);
        test_stop(selector, &user_data);

        lol_html_selector_free(selector);
    }

    {
        note("NamespaceURI");

        const char *selector_str = "script";

        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        test_element_ns_is_html(selector, &user_data);
        test_element_ns_is_svg(selector, &user_data);

        lol_html_selector_free(selector);
    }

    {
        note("EndTagChange");

        const char *selector_str = "div";
        lol_html_selector_t *selector = lol_html_selector_parse(
            selector_str,
            strlen(selector_str)
        );

        lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

        lol_ok(lol_html_rewriter_builder_add_element_content_handlers(
            builder,
            selector,
            modify_element_end_tag_name_outer,
            NULL,
            NULL,
            NULL,
            NULL,
            NULL
        ));

        const char *input = "<div>42</div><div>some data</div>";
        run_rewriter(builder, input, modify_element_end_tag, &user_data);
    }
}
