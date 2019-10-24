#include <stdlib.h>
#include "deps/picotest/picotest.h"
#include "test_util.h"

void output_sink_stub(const char *chunk, size_t chunk_len, void *user_data) {
    (void)(chunk);
    (void)(chunk_len);
    (void)(user_data);
}

cool_thing_rewriter_directive_t get_and_free_empty_element_attribute(
    cool_thing_element_t *element,
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
