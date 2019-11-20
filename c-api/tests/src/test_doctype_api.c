#include "../../include/lol_html.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static int EXPECTED_USER_DATA = 42;

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_doctype_fields_output_sink,
    "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
)

static lol_html_rewriter_directive_t get_doctype_fields(
    lol_html_doctype_t *doctype,
    void *user_data
) {
    UNUSED(user_data);

    note("Fields");

    lol_html_str_t *name = lol_html_doctype_name_get(doctype);
    lol_html_str_t *public_id = lol_html_doctype_public_id_get(doctype);
    lol_html_str_t *system_id = lol_html_doctype_system_id_get(doctype);

    str_eq(name, "math");
    ok(public_id == NULL);
    str_eq(system_id, "http://www.w3.org/Math/DTD/mathml1/mathml.dtd");

    lol_html_str_free(*name);
    lol_html_str_free(*system_id);

    return LOL_HTML_CONTINUE;
}

static void test_get_doctype_fields(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        &get_doctype_fields,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(
        builder,
        "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
        get_doctype_fields_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
EXPECT_OUTPUT(
    get_user_data_output_sink,
    "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
)

static lol_html_rewriter_directive_t get_user_data(
    lol_html_doctype_t *doctype,
    void *user_data
) {
    note("User data");
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    note("Set doctype user data");
    lol_html_doctype_user_data_set(doctype, user_data);


    note("Get doctype user data");

    int doctype_user_data = *(int*)lol_html_doctype_user_data_get(doctype);

    ok(doctype_user_data == EXPECTED_USER_DATA);

    return LOL_HTML_CONTINUE;
}

static void test_get_user_data(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        &get_user_data,
        user_data,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(
        builder,
        "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
        get_user_data_output_sink,
        user_data
    );
}

//-------------------------------------------------------------------------
static lol_html_rewriter_directive_t stop_rewriting(
    lol_html_doctype_t *doctype,
    void *user_data
) {
    UNUSED(doctype);
    UNUSED(user_data);

    note("Stop rewriting");

    return LOL_HTML_STOP;
}

static void test_stop(void *user_data) {
    lol_html_rewriter_builder_t *builder = lol_html_rewriter_builder_new();

    lol_html_rewriter_builder_add_document_content_handlers(
        builder,
        &stop_rewriting,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    expect_stop(builder, "<!doctype>", user_data);
}

void test_doctype_api() {
    int user_data = 42;

    test_get_doctype_fields(&user_data);
    test_get_user_data(&user_data);
    test_stop(&user_data);
}
