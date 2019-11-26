#include <string.h>

#include "../../include/lol_html.h"
#include "deps/picotest/picotest.h"
#include "tests.h"
#include "test_util.h"

void test_unsupported_selector() {
    const char *selector_str = "p:last-child";
    lol_html_selector_t *selector = lol_html_selector_parse(selector_str, strlen(selector_str));

    ok(selector == NULL);

    lol_html_str_t *msg = lol_html_take_last_error();

    str_eq(msg, "Unsupported pseudo-class or pseudo-element in selector.");

    lol_html_str_free(*msg);
}
