#ifndef COOL_THING_H
#define COOL_THING_H

#if defined(__cplusplus)
extern "C" {
#endif

#include <stddef.h>
#include <stdbool.h>

// NOTE: all functions that accept pointers will panic abort the thread
// if NULL pointer is passed (with an exception for the cases where
// explicitly stated that function can accept NULL pointers).

// NOTE: all UTF8-strings passed to the API functions allow interior '\0's
// and their length determined by the corresponding length parameter only.

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
typedef struct cool_thing_AttributesIterator cool_thing_attributes_iterator_t;
typedef struct cool_thing_Attribute cool_thing_attribute_t;

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

// A fat pointer to text chunk content.
//
// The difference between this struct and `cool_thing_str_t` is
// that text chunk content shouldn't be deallocated manually via
// `cool_thing_str_free` method call. Instead the pointer becomes
// invalid ones related `cool_thing_text_chunk_t` struct goes out
// of scope.
typedef struct {
    // String data pointer.
    const char *data;

    // The length of the string in bytes.
    size_t len;
} cool_thing_text_chunk_content_t;

// Utilities
//---------------------------------------------------------------------

// Frees the memory held by the library-allocated string.
void cool_thing_str_free(cool_thing_str_t str);

// Returns the last error message and resets last error to NULL.
//
// Return NULL if there was no error.
cool_thing_str_t *cool_thing_take_last_error();

// Creates new HTML rewriter builder.
cool_thing_rewriter_builder_t *cool_thing_rewriter_builder_new();

// Content handlers
//---------------------------------------------------------------------
// Rewriter directive that should be returned from each content handler.
// If COOL_THING_STOP directive is returned then rewriting stops immidiately
// and `write()` or `end()` methods of the rewriter return an error code.
typedef enum {
    COOL_THING_CONTINUE,
    COOL_THING_STOP
} cool_thing_rewriter_directive_t;

typedef cool_thing_rewriter_directive_t (*cool_thing_doctype_handler_t)(
    cool_thing_doctype_t *doctype,
    void *user_data
);

typedef cool_thing_rewriter_directive_t (*cool_thing_comment_handler_t)(
    cool_thing_comment_t *comment,
    void *user_data
);

typedef cool_thing_rewriter_directive_t (*cool_thing_text_handler_handler_t)(
    cool_thing_text_chunk_t *chunk,
    void *user_data
);

typedef cool_thing_rewriter_directive_t (*cool_thing_element_handler_t)(
    cool_thing_element_t *element,
    void *user_data
);

// Rewriter builder
//---------------------------------------------------------------------

// Adds document-level content handlers to the builder.
//
// If a particular handler is not required then NULL can be passed
// instead. Don't use stub handlers in this case as this affects
// performance - rewriter skips parsing of the content that doesn't
// need to be processed.
//
// Each handler can optionally have associated user data which will be
// passed to the handler on each invocation along with the rewritable
// unit argument.
//
// If any of handlers return COOL_THING_STOP directive is then rewriting
// stops immidiately and `write()` or `end()` of the rewriter methods
// return an error code.
//
// WARNING: Pointers passed to handlers are valid only during the
// handler execution. So they should never be leaked outside of handlers.
void cool_thing_rewriter_builder_add_document_content_handlers(
    cool_thing_rewriter_builder_t *builder,
    cool_thing_doctype_handler_t doctype_handler,
    void *doctype_handler_user_data,
    cool_thing_comment_handler_t comment_handler,
    void *comment_handler_user_data,
    cool_thing_text_handler_handler_t text_handler,
    void *text_handler_user_data
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
// Each handler can optionally have associated user data which will be
// passed to the handler on each invocation along with the rewritable
// unit argument.
//
// If any of handlers return COOL_THING_STOP directive is then rewriting
// stops immidiately and `write()` or `end()` of the rewriter methods
// return an error code.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
//
// WARNING: Pointers passed to handlers are valid only during the
// handler execution. So they should never be leaked outside of handlers.
int cool_thing_rewriter_builder_add_element_content_handlers(
    cool_thing_rewriter_builder_t *builder,
    const char *selector,
    size_t selector_len,
    cool_thing_element_handler_t element_handler,
    void *element_handler_user_data,
    cool_thing_comment_handler_t comment_handler,
    void *comment_handler_user_data,
    cool_thing_text_handler_handler_t text_handler,
    void *text_handler_user_data
);

// Frees the memory held by the builder.
//
// Note that builder can be freed before any rewriters constructed from
// it if it's not intended to be used anymore.
void cool_thing_rewriter_builder_free(cool_thing_rewriter_builder_t *builder);


// Rewriter
//---------------------------------------------------------------------

// Builds HTML-rewriter out of the provided builder. Can be called
// multiple times to construct different rewriters from the same
// builder.
//
// `output_sink` receives a zero-length chunk on the end of the output.
//
// `output_sink` can optionally have associated user data that will
// be passed to handler on each invocation along with other arguments.
//
// In case of an error the function returns a NULL pointer.
cool_thing_rewriter_t *cool_thing_rewriter_build(
    cool_thing_rewriter_builder_t *builder,
    const char *encoding,
    size_t encoding_len,
    size_t buffer_capacity,
    void (*output_sink)(const char *chunk, size_t chunk_len, void *user_data),
    void *output_sink_user_data
);

// Write HTML chunk to rewriter.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
//
// WARNING: if this function errors the rewriter gets into the unrecovarable state,
// so any further attempts to use the rewriter will cause a thread panic.
int cool_thing_rewriter_write(
    cool_thing_rewriter_t *rewriter,
    const char *chunk,
    size_t chunk_len
);

// Completes rewriting and flushes the remaining output.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
//
// WARNING: if this function errors the rewriter gets into the unrecovarable state,
// so any further attempts to use the rewriter will cause a thread panic.
int cool_thing_rewriter_end(cool_thing_rewriter_t *rewriter);

// Frees the memory held by the rewriter.
void cool_thing_rewriter_free(cool_thing_rewriter_t *rewriter);

// Doctype
//---------------------------------------------------------------------

// Returns doctype's name.
//
// Returns NULL if the doctype doesn't have a name.
cool_thing_str_t *cool_thing_doctype_name_get(const cool_thing_doctype_t *doctype);

// Returns doctype's PUBLIC identifier.
//
// Returns NULL if the doctype doesn't have a PUBLIC identifier.
cool_thing_str_t *cool_thing_doctype_public_id_get(const cool_thing_doctype_t *doctype);

// Returns doctype's SYSTEM identifier.
//
// Returns NULL if the doctype doesn't have a SYSTEM identifier.
cool_thing_str_t *cool_thing_doctype_system_id_get(const cool_thing_doctype_t *doctype);

// Attaches custom user data to the doctype.
//
// The same doctype can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some processing
// state on the doctype, so it can be shared between handlers.
void cool_thing_doctype_user_data_set(
    const cool_thing_doctype_t *doctype,
    void *user_data
);

// Returns user data attached to the doctype.
void *cool_thing_doctype_user_data_get(const cool_thing_doctype_t *doctype);

// Comment
//---------------------------------------------------------------------

// Returns comment text.
cool_thing_str_t cool_thing_comment_text_get(const cool_thing_comment_t *comment);

// Sets comment text.
//
// Text should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_comment_text_set(
    cool_thing_comment_t *comment,
    const char *text,
    size_t text_len
);

// Inserts the content string before the comment either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_comment_before(
    cool_thing_comment_t *comment,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string after the comment either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_comment_after(
    cool_thing_comment_t *comment,
    const char *content,
    size_t content_len,
    bool is_html
);

// Replace the comment with the content of the string which is interpreted
// either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_comment_replace(
    cool_thing_comment_t *comment,
    const char *content,
    size_t content_len,
    bool is_html
);

// Removes the comment.
void cool_thing_comment_remove(cool_thing_comment_t *comment);

// Returns `true` if the comment has been removed.
bool cool_thing_comment_is_removed(const cool_thing_comment_t *comment);

// Attaches custom user data to the comment.
//
// The same comment can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some
// processing state on the comment, so it can be shared between handlers.
void cool_thing_comment_user_data_set(
    const cool_thing_comment_t *comment,
    void *user_data
);

// Returns user data attached to the comment.
void *cool_thing_comment_user_data_get(const cool_thing_comment_t *comment);


// Text chunk
//---------------------------------------------------------------------

// Returns a fat pointer to the UTF8 representation of content of the chunk.
//
// If the chunk is last in the current text node then content can be an empty string.
//
// WARNING: The pointer is valid only during the handler execution and
// should never be leaked outside of handlers.
cool_thing_text_chunk_content_t cool_thing_text_chunk_content_get(
    const cool_thing_text_chunk_t *chunk
);

// Returns `true` if the chunk is last in the current text node.
bool cool_thing_text_chunk_is_last_in_text_node(const cool_thing_text_chunk_t *chunk);

// Inserts the content string before the text chunk either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_text_chunk_before(
    cool_thing_text_chunk_t *chunk,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string after the text chunk either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_text_chunk_after(
    cool_thing_text_chunk_t *chunk,
    const char *content,
    size_t content_len,
    bool is_html
);

// Replace the text chunk with the content of the string which is interpreted
// either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_text_chunk_replace(
    cool_thing_text_chunk_t *chunk,
    const char *content,
    size_t content_len,
    bool is_html
);

// Removes the text chunk.
void cool_thing_text_chunk_remove(cool_thing_text_chunk_t *chunk);

// Returns `true` if the text chunk has been removed.
bool cool_thing_text_chunk_is_removed(const cool_thing_text_chunk_t *chunk);

// Attaches custom user data to the text chunk.
//
// The same text chunk can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some processing
// state on the chunk, so it can be shared between handlers.
void cool_thing_text_chunk_user_data_set(
    const cool_thing_text_chunk_t *chunk,
    void *user_data
);

// Returns user data attached to the text chunk.
void *cool_thing_text_chunk_user_data_get(const cool_thing_text_chunk_t *chunk);


// Element
//---------------------------------------------------------------------

// Returns the tag name of the element.
cool_thing_str_t cool_thing_element_tag_name_get(const cool_thing_element_t *element);

// Sets the tag name of the element.
//
// Name should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_tag_name_set(
    cool_thing_element_t *element,
    const char *name,
    size_t name_len
);

// Returns the iterator over the element attributes.
//
// WARNING: The iterator is valid only during the handler execution and
// should never be leaked outside of it.
//
// Use `cool_thing_attributes_iterator_free` function to deallocate
// returned iterator.
cool_thing_attributes_iterator_t *cool_thing_attributes_iterator_get(
    const cool_thing_element_t *element
);

// Frees the memory held by the attribute iterator.
void cool_thing_attributes_iterator_free(cool_thing_attributes_iterator_t *iterator);

// Advances the iterator and returns next attribute.
//
// Returns NULL if iterator has been exhausted.
//
// WARNING: Returned attribute is valid only during the handler
// execution and should never be leaked outside of it.
const cool_thing_attribute_t *cool_thing_attributes_iterator_next(
    cool_thing_attributes_iterator_t *iterator
);

// Returns the attribute name.
cool_thing_str_t cool_thing_attribute_name_get(const cool_thing_attribute_t *attribute);

// Returns the attribute value.
cool_thing_str_t cool_thing_attribute_value_get(const cool_thing_attribute_t *attribute);

// Returns the attribute value or NULL if attribute with the given name
// doesn't exist on the element.
//
// Name should be a valid UTF8-string.
//
// If the provided name is invalid UTF8-string the function returns NULL as well.
// Therefore one should always check `cool_thing_take_last_error` result after the call.
cool_thing_str_t *cool_thing_element_get_attribute(
    const cool_thing_element_t *element,
    const char *name,
    size_t name_len
);

// Returns 1 if element has attribute with the given name, and 0 otherwise.
// Returns -1 in case of an error.
//
// Name should be a valid UTF8-string.
int cool_thing_element_has_attribute(
    const cool_thing_element_t *element,
    const char *name,
    size_t name_len
);

// Updates the attribute value if attribute with the given name already exists on
// the element, or creates adds new attribute with given name and value otherwise.
//
// Name and value should be valid UTF8-strings.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_set_attribute(
    cool_thing_element_t *element,
    const char *name,
    size_t name_len,
    const char *value,
    size_t value_len
);

// Removes the attribute with the given name from the element.
//
// Name should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_remove_attribute(
    cool_thing_element_t *element,
    const char *name,
    size_t name_len
);

// Inserts the content string before the element either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_before(
    cool_thing_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string right after the element's start tag
// either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_prepend(
    cool_thing_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string right before the element's end tag
// either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_append(
    cool_thing_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string right after the element's end tag as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_after(
    cool_thing_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Sets either text or HTML inner content of the element.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_set_inner_content(
    cool_thing_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Replaces the element with the provided text or HTML content.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `cool_thing_take_last_error` function.
int cool_thing_element_replace(
    cool_thing_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Removes the element.
void cool_thing_element_remove(const cool_thing_element_t *element);

// Removes the element, but leaves its inner content intact.
void cool_thing_element_remove_and_keep_content(const cool_thing_element_t *element);

// Returns `true` if the element has been removed.
bool cool_thing_element_is_removed(const cool_thing_element_t *element);

// Attaches custom user data to the element.
//
// The same element can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some processing
// state on the element, so it can be shared between handlers.
void cool_thing_element_user_data_set(
    const cool_thing_element_t *element,
    void *user_data
);

// Returns user data attached to the text chunk.
void *cool_thing_element_user_data_get(const cool_thing_element_t *element);

#if defined(__cplusplus)
}  // extern C
#endif

#endif // COOL_THING_H