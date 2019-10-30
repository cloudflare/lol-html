#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

static int EXPECTED_USER_DATA = 42;

static cool_thing_rewriter_directive_t doctype_handler(
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
    ok(*(int*)user_data == EXPECTED_USER_DATA);

    note("Set doctype user data");
    cool_thing_doctype_user_data_set(doctype, user_data);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t user_data_get(
    cool_thing_doctype_t *doctype,
    void *user_data
) {
    UNUSED(user_data);

    note("Get doctype user data");

    int doctype_user_data = *(int*)cool_thing_doctype_user_data_get(doctype);

    ok(doctype_user_data == EXPECTED_USER_DATA);

    return COOL_THING_CONTINUE;
}

static cool_thing_rewriter_directive_t stop_rewriting(
    cool_thing_doctype_t *doctype,
    void *user_data
) {
    UNUSED(doctype);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

EXPECT_OUTPUT(
    output_sink,
    "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
    &EXPECTED_USER_DATA,
    sizeof(EXPECTED_USER_DATA)
)

static void test_rewrite(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        &doctype_handler,
        user_data,
        NULL,
        NULL,
        NULL,
        NULL
    );

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        &user_data_get,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

    run_rewriter(
        builder,
        "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">",
        output_sink,
        user_data
    );
}

static void test_stop(void *user_data) {
    cool_thing_rewriter_builder_t *builder = cool_thing_rewriter_builder_new();

    cool_thing_rewriter_builder_add_document_content_handlers(
        builder,
        &stop_rewriting,
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

    test_rewrite(&user_data);
    test_stop(&user_data);
}

