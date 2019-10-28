#include <string.h>

#include "../../include/cool_thing.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

void test_unsupported_selector() {
    const char *selector_str = "p:last-child";
    cool_thing_selector_t *selector = cool_thing_selector_parse(selector_str, strlen(selector_str));

    ok(selector == NULL);

    cool_thing_str_t *msg = cool_thing_take_last_error();

    str_eq(msg, "Unsupported pseudo-class or pseudo-element in selector.");

    cool_thing_str_free(*msg);
}
