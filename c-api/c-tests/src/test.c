#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include "deps/picotest/picotest.h"
#include "../../include/lol_html.h"

#include "tests.h"

int run_tests() {
    subtest("Unsupported selector", test_unsupported_selector);
    subtest("Non-ASCII encoding", test_non_ascii_encoding);
    subtest("Doctype API", test_doctype_api);
    subtest("Comment API", test_comment_api);
    subtest("Text chunk API", test_text_chunk_api);
    subtest("Element API", element_api_test);
    subtest("Document end API", document_end_api_test);
    subtest("Memory limiting", test_memory_limiting);
    int res = done_testing();
    if (res) {
        fprintf(stderr, "\nSome tests have failed\n");
    }
    return res;
}
