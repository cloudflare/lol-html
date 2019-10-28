#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

cool_thing_rewriter_directive_t test_doctype_api_doctype_handler(
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

cool_thing_rewriter_directive_t test_doctype_api_user_data_get(
    cool_thing_doctype_t *doctype,
    void *user_data
) {
    UNUSED(user_data);

    note("Get doctype user data");

    int doctype_user_data = *(int*)cool_thing_doctype_user_data_get(doctype);

    ok(doctype_user_data == 42);

    return COOL_THING_CONTINUE;
}

cool_thing_rewriter_directive_t test_doctype_api_stop_rewriting (
    cool_thing_doctype_t *doctype,
    void *user_data
) {
    UNUSED(doctype);
    UNUSED(user_data);

    note("Stop rewriting");

    return COOL_THING_STOP;
}

EXPECT_OUTPUT(
    test_doctype_api_output,
    "<!DOCTYPE math SYSTEM \"http://www.w3.org/Math/DTD/mathml1/mathml.dtd\">"
)

void test_doctype_api() {
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

