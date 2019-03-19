#ifndef COOL_THING_H
#define COOL_THING_H

#if defined(__cplusplus)
extern "C" {
#endif

#include <stddef.h>

// Opaque structures used by the rewriter.
// WARNING: these structures should never be deallocated by the C code.
// There are appropriate methods exposed that take care of these structures
// deallocation.
typedef struct cool_thing_HtmlRewriterBuilder cool_thing_rewriter_builder_t;
typedef struct cool_thing_HtmlRewriter cool_thing_rewriter_t;
typedef struct cool_thing_Doctype cool_thing_doctype_t;
typedef struct cool_thing_Comment cool_thing_comment_t;
typedef struct cool_thing_TextChunk cool_thing_text_chunk_t;
typedef struct cool_thing_Element cool_thing_element_t;

// Library-allocated UTF8 string fat pointer.
//
// The string is not NULL-terminated.
//
// Should NEVER be deallocated in the C code. Use special `cool_thing_str_free`
// function instead.
typedef struct {
    // String data pointer.
    const char *data;

    // The length of the string in bytes.
    size_t len;
} cool_thing_str_t;

// NOTE: all functions that accept pointers will panic abort the thread
// if NULL pointer is passed (with an exception for the cases where
// explicitly stated that function can accept NULL pointers).

// Frees the memory held by the library-allocated string.
void cool_thing_str_free(cool_thing_str_t *str);

// Returns the last error message.
//
// Return NULL if there was no error.
cool_thing_str_t *cool_thing_get_last_error();

// Creates new HTML rewriter builder.
cool_thing_rewriter_builder_t *cool_thing_rewriter_builder_new();

// Adds document-level content handlers to the builder.
//
// If a particular handler is not required then NULL can be passed
// instead. Don't use stub handlers in this case as this affects
// performance - rewriter skips parsing of the content that doesn't
// need to be processed.
//
// Returns 0 in case of success and -1 othewise. The actual error message
// can be obtained using `cool_thing_get_last_error` function.
void cool_thing_rewriter_builder_add_document_content_handlers(
    cool_thing_rewriter_builder_t *builder,
    void (*doctype_handler)(cool_thing_doctype_t *doctype),
    void (*comment_handler)(cool_thing_comment_t *comment),
    void (*text_handler)(cool_thing_text_chunk_t *chunk)
);

// Adds element content handlers to the builder for the
// given CSS selector.
//
// Selector should be a valid UTF8-string.
//
// If a particular handler is not required then NULL can be passed
// instead. Don't use stub handlers in this case as this affects
// performance - rewriter skips parsing of the content that doesn't
// need to be processed.
//
// Returns 0 in case of success and -1 othewise. The actual error message
// can be obtained using `cool_thing_get_last_error` function.
int cool_thing_rewriter_builder_add_element_content_handlers(
    cool_thing_rewriter_builder_t *builder,
    const char *selector,
    size_t selector_len,
    void (*element_handler)(cool_thing_element_t *element),
    void (*comment_handler)(cool_thing_comment_t *comment),
    void (*text_handler)(cool_thing_text_chunk_t *chunk)
);

// Builds HTML-rewriter out of the provided builder.
//
// This function deallocates provided builder, so it can't be used after the call.
//
// In case of an error the function returns a NULL pointer.
cool_thing_rewriter_t *cool_thing_rewriter_build(
    cool_thing_rewriter_builder_t *builder,
    const char *encoding,
    size_t encoding_len,
    void (*output_sink)(const char *chunk, size_t chunk_len)
);

// Write HTML chunk to rewriter.
//
// Returns 0 in case of success and -1 othewise. The actual error message
// can be obtained using `cool_thing_get_last_error` function.
int cool_thing_rewriter_write(
    cool_thing_rewriter_t *rewriter,
    const char *chunk,
    size_t chunk_len
);

// Completes rewriting, flushes the remaining output and deallocates the rewriter.
//
// Returns 0 in case of success and -1 othewise. The actual error message
// can be obtained using `cool_thing_get_last_error` function.
int cool_thing_rewriter_end(cool_thing_rewriter_t *rewriter);

// Returns doctype's name.
//
// Returns NULL if doctype doesn't have a name.
cool_thing_str_t *cool_thing_doctype_name_get(const cool_thing_doctype_t *doctype);

// Returns doctype's PUBLIC identifier.
//
// Returns NULL if doctype doesn't have a PUBLIC identifier.
cool_thing_str_t *cool_thing_doctype_public_id_get(const cool_thing_doctype_t *doctype);

// Returns doctype's SYSTEM identifier.
//
// Returns NULL if doctype doesn't have a SYSTEM identifier.
cool_thing_str_t *cool_thing_doctype_system_id_get(const cool_thing_doctype_t *doctype);

// Returns comment text.
cool_thing_str_t *cool_thing_comment_text_get(const cool_thing_comment_t *comment);

// Sets comment text.
//
// Text should be a valid UTF8-string.
cool_thing_str_t *cool_thing_comment_text_set(
    cool_thing_comment_t *comment,
    const char *text,
    size_t text_len
);

#if defined(__cplusplus)
}  // extern C
#endif

#endif // COOL_THING_H