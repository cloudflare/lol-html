#ifndef TESTS_H
#define TESTS_H

#include "../../include/lol_html.h"

void test_unsupported_selector();
void test_non_ascii_encoding();
void test_doctype_api();
void test_comment_api();
void test_text_chunk_api();
void element_api_test();
void test_memory_limiting();

#endif // TESTS_H
