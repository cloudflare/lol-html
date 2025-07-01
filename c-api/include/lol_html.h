#ifndef LOL_HTML_H
#define LOL_HTML_H

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
typedef struct lol_html_HtmlRewriterBuilder lol_html_rewriter_builder_t;
typedef struct lol_html_HtmlRewriter lol_html_rewriter_t;
typedef struct lol_html_Doctype lol_html_doctype_t;
typedef struct lol_html_DocumentEnd lol_html_doc_end_t;
typedef struct lol_html_EndTag lol_html_end_tag_t;
typedef struct lol_html_Comment lol_html_comment_t;
typedef struct lol_html_TextChunk lol_html_text_chunk_t;
typedef struct lol_html_Element lol_html_element_t;
typedef struct lol_html_AttributesIterator lol_html_attributes_iterator_t;
typedef struct lol_html_Attribute lol_html_attribute_t;
typedef struct lol_html_Selector lol_html_selector_t;
typedef struct lol_html_CStreamingHandlerSink lol_html_streaming_sink_t;

// Library-allocated UTF8 string fat pointer.
//
// The string is not NULL-terminated.
//
// Should NEVER be deallocated in the C code. Use special `lol_html_str_free`
// function instead.
typedef struct {
    // String data pointer.
    const char *data;

    // The length of the string in bytes.
    size_t len;
} lol_html_str_t;

// A fat pointer to text chunk content.
//
// The difference between this struct and `lol_html_str_t` is
// that text chunk content shouldn't be deallocated manually via
// `lol_html_str_free` method call. Instead the pointer becomes
// invalid ones related `lol_html_text_chunk_t` struct goes out
// of scope.
typedef struct {
    // String data pointer.
    const char *data;

    // The length of the string in bytes.
    size_t len;
} lol_html_text_chunk_content_t;

// Utilities
//---------------------------------------------------------------------

// Frees the memory held by the library-allocated string.
//
// This is valid to call even if `str.data == NULL` (it does nothing, like `free(NULL)`).
void lol_html_str_free(lol_html_str_t str);

// Returns the last error message and resets last error to NULL.
//
// The `data` field will be NULL if there was no error.
lol_html_str_t lol_html_take_last_error();

// Creates new HTML rewriter builder.
lol_html_rewriter_builder_t *lol_html_rewriter_builder_new();

// Content handlers
//---------------------------------------------------------------------
// Rewriter directive that should be returned from each content handler.
// If LOL_HTML_STOP directive is returned then rewriting stops immediately
// and `write()` or `end()` methods of the rewriter return an error code.
typedef enum {
    LOL_HTML_CONTINUE,
    LOL_HTML_STOP
} lol_html_rewriter_directive_t;

typedef lol_html_rewriter_directive_t (*lol_html_doctype_handler_t)(
    lol_html_doctype_t *doctype,
    void *user_data
);

typedef lol_html_rewriter_directive_t (*lol_html_comment_handler_t)(
    lol_html_comment_t *comment,
    void *user_data
);

typedef lol_html_rewriter_directive_t (*lol_html_text_handler_handler_t)(
    lol_html_text_chunk_t *chunk,
    void *user_data
);

typedef lol_html_rewriter_directive_t (*lol_html_element_handler_t)(
    lol_html_element_t *element,
    void *user_data
);

typedef lol_html_rewriter_directive_t (*lol_html_doc_end_handler_t)(
    lol_html_doc_end_t *doc_end,
    void *user_data
);

typedef lol_html_rewriter_directive_t (*lol_html_end_tag_handler_t)(
    lol_html_end_tag_t *end_tag,
    void *user_data
);

// `size_t` byte offsets from the start of the input document
typedef struct lol_html_SourceLocationBytes {
    size_t start;
    size_t end;
} lol_html_source_location_bytes_t;

// For use with streaming content handlers.
//
// Safety: the user data and the callbacks must be safe to use from a different thread (e.g. can't rely on thread-local storage).
// It doesn't have to be `Sync`, it will be used only by one thread at a time.
//
// Handler functions copy this struct. It can (and should) be created on the stack.
typedef struct lol_html_CStreamingHandler {
    // Anything you like
    void *user_data;
    // Called when the handler is supposed to produce its output. Return `0` for success.
    // The `sink` argument is guaranteed non-`NULL`. It is valid only for the duration of this call, and can only be used on the same thread.
    // The sink is for [`lol_html_streaming_sink_write_str`] and [`lol_html_streaming_sink_write_utf8_chunk`].
    // `user_data` comes from this struct.
    // `write_all_callback` must not be `NULL`.
    int (*write_all_callback)(lol_html_streaming_sink_t *sink, void *user_data);
    // Called exactly once, after the last use of this handler.
    // `user_data` comes from this struct.
    // May be `NULL`.
    void (*drop_callback)(void *user_data);
    // *Always* initialize to `NULL`.
    void *reserved;
} lol_html_streaming_handler_t;

// Selector
//---------------------------------------------------------------------

// Parses given CSS selector string.
//
// Returns NULL if parsing error occurs. The actual error message
// can be obtained using `lol_html_take_last_error` function.
//
// WARNING: Selector SHOULD NOT be deallocated if there are any active rewriter
// builders that accepted it as an argument to `lol_html_rewriter_builder_add_element_content_handlers()`
// method. Deallocate all dependant rewriter builders first and then
// use `lol_html_selector_free` function to free the selector.
lol_html_selector_t *lol_html_selector_parse(
    const char *selector,
    size_t selector_len
);

// Frees the memory held by the parsed selector object.
void lol_html_selector_free(lol_html_selector_t *selector);


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
// If any of handlers return LOL_HTML_STOP directive then rewriting
// stops immediately and `write()` or `end()` of the rewriter methods
// return an error code.
//
// WARNING: Pointers passed to handlers are valid only during the
// handler execution. So they should never be leaked outside of handlers.
void lol_html_rewriter_builder_add_document_content_handlers(
    lol_html_rewriter_builder_t *builder,
    lol_html_doctype_handler_t doctype_handler,
    void *doctype_handler_user_data,
    lol_html_comment_handler_t comment_handler,
    void *comment_handler_user_data,
    lol_html_text_handler_handler_t text_handler,
    void *text_handler_user_data,
    lol_html_doc_end_handler_t doc_end_handler,
    void *doc_end_user_data
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
// If any of handlers return LOL_HTML_STOP directive then rewriting
// stops immediately and `write()` or `end()` of the rewriter methods
// return an error code.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
//
// WARNING: Pointers passed to handlers are valid only during the
// handler execution. So they should never be leaked outside of handlers.
int lol_html_rewriter_builder_add_element_content_handlers(
    lol_html_rewriter_builder_t *builder,
    const lol_html_selector_t *selector,
    lol_html_element_handler_t element_handler,
    void *element_handler_user_data,
    lol_html_comment_handler_t comment_handler,
    void *comment_handler_user_data,
    lol_html_text_handler_handler_t text_handler,
    void *text_handler_user_data
);

// Frees the memory held by the builder.
//
// Note that builder can be freed before any rewriters constructed from
// it if it's not intended to be used anymore.
void lol_html_rewriter_builder_free(lol_html_rewriter_builder_t *builder);


// Rewriter
//---------------------------------------------------------------------

// Memory management settings for the rewriter.
typedef struct {
    // Preallocated size of the parsing buffer.
    //
    // Can be set to 0. In this case rewriter won't consume any memory initially,
    // though there might be a performance penalty due to later reallocations.
    size_t preallocated_parsing_buffer_size;
    // Maximum amount of memory to be used by a rewriter.
    //
    // `lol_html_rewriter_write` and `lol_html_rewriter_end` will return an error
    // if this limit is exceeded.
    size_t max_allowed_memory_usage;
} lol_html_memory_settings_t;

// Builds HTML-rewriter out of the provided builder. Can be called
// multiple times to construct different rewriters from the same
// builder.
//
// `output_sink` receives a zero-length chunk on the end of the output.
//
// `output_sink` can optionally have associated user data that will
// be passed to handler on each invocation along with other arguments.
//
// `strict` mode will bail out from tokenization process in cases when
// there is no way to determine correct parsing context. Recommended
// setting for safety reasons.
//
// In case of an error the function returns a NULL pointer.
lol_html_rewriter_t *lol_html_rewriter_build(
    lol_html_rewriter_builder_t *builder,
    const char *encoding,
    size_t encoding_len,
    lol_html_memory_settings_t memory_settings,
    void (*output_sink)(const char *chunk, size_t chunk_len, void *user_data),
    void *output_sink_user_data,
    bool strict
);

lol_html_rewriter_t *unstable_lol_html_rewriter_build_with_esi_tags(
    lol_html_rewriter_builder_t *builder,
    const char *encoding,
    size_t encoding_len,
    lol_html_memory_settings_t memory_settings,
    void (*output_sink)(const char *chunk, size_t chunk_len, void *user_data),
    void *output_sink_user_data,
    bool strict
);

// Write HTML chunk to rewriter.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
//
// WARNING: if this function errors the rewriter gets into the unrecoverable state,
// so any further attempts to use the rewriter will cause a thread panic.
int lol_html_rewriter_write(
    lol_html_rewriter_t *rewriter,
    const char *chunk,
    size_t chunk_len
);

// Completes rewriting and flushes the remaining output.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
//
// WARNING: after calling this function, further attempts to use the rewriter
// (other than `lol_html_rewriter_free`) will cause a thread panic.
int lol_html_rewriter_end(lol_html_rewriter_t *rewriter);

// Frees the memory held by the rewriter.
void lol_html_rewriter_free(lol_html_rewriter_t *rewriter);

// Doctype
//---------------------------------------------------------------------

// Returns doctype's name.
//
// The `data` field will be NULL if the doctype doesn't have a name.
lol_html_str_t lol_html_doctype_name_get(const lol_html_doctype_t *doctype);

// Returns doctype's PUBLIC identifier.
//
// The `data` field will be NULL if the doctype doesn't have a PUBLIC identifier.
lol_html_str_t lol_html_doctype_public_id_get(const lol_html_doctype_t *doctype);

// Returns doctype's SYSTEM identifier.
//
// The `data` field will be NULL if the doctype doesn't have a SYSTEM identifier.
lol_html_str_t lol_html_doctype_system_id_get(const lol_html_doctype_t *doctype);

// Attaches custom user data to the doctype.
//
// The same doctype can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some processing
// state on the doctype, so it can be shared between handlers.
void lol_html_doctype_user_data_set(
    const lol_html_doctype_t *doctype,
    void *user_data
);

// Returns user data attached to the doctype.
void *lol_html_doctype_user_data_get(const lol_html_doctype_t *doctype);

// Removes the doctype.
void lol_html_doctype_remove(lol_html_doctype_t *doctype);

// Returns `true` if the doctype has been removed.
bool lol_html_doctype_is_removed(const lol_html_doctype_t *doctype);

// Returns [`SourceLocationBytes`].
//
// `doctype` must be valid and non-`NULL`.
lol_html_source_location_bytes_t lol_html_doctype_source_location_bytes(lol_html_doctype_t *doctype);

// Comment
//---------------------------------------------------------------------

// Returns comment text.
lol_html_str_t lol_html_comment_text_get(const lol_html_comment_t *comment);

// Sets comment text.
//
// Text should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_comment_text_set(
    lol_html_comment_t *comment,
    const char *text,
    size_t text_len
);

// Inserts the content string before the comment either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_comment_before(
    lol_html_comment_t *comment,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string after the comment either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_comment_after(
    lol_html_comment_t *comment,
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
// can be obtained using `lol_html_take_last_error` function.
int lol_html_comment_replace(
    lol_html_comment_t *comment,
    const char *content,
    size_t content_len,
    bool is_html
);

// Removes the comment.
// `comment` must be valid and non-`NULL`.
//
// Calls [`Comment::remove`].
void lol_html_comment_remove(lol_html_comment_t *comment);

// Returns `true` if the comment has been removed.
bool lol_html_comment_is_removed(const lol_html_comment_t *comment);

// Attaches custom user data to the comment.
//
// The same comment can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some
// processing state on the comment, so it can be shared between handlers.
void lol_html_comment_user_data_set(
    const lol_html_comment_t *comment,
    void *user_data
);

// Returns user data attached to the comment.
void *lol_html_comment_user_data_get(const lol_html_comment_t *comment);

// Returns [`SourceLocationBytes`].
//
// `comment` must be valid and non-`NULL`.
lol_html_source_location_bytes_t lol_html_comment_source_location_bytes(lol_html_comment_t *comment);

// Element
//---------------------------------------------------------------------

// Returns the tag name of the element.
lol_html_str_t lol_html_element_tag_name_get(const lol_html_element_t *element);

// Returns the tag name of the element, preserving its case.
lol_html_str_t lol_html_element_tag_name_get_preserve_case(const lol_html_element_t *element);

// Sets the tag name of the element.
//
// Name should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_element_tag_name_set(
    lol_html_element_t *element,
    const char *name,
    size_t name_len
);

// Whether the tag syntactically ends with `/>`. In HTML content this is purely a decorative, unnecessary, and has no effect of any kind.
//
// The `/>` syntax only affects parsing of elements in foreign content (SVG and MathML).
// It will never close any HTML tags that aren't already defined as void in HTML.
//
// This function only reports the parsed syntax, and will not report which elements are actually void in HTML.
// Use `lol_html_element_can_have_content` to check if the element is non-void.
//
// If the `/` is part of an unquoted attribute, it's not parsed as the self-closing syntax.
bool lol_html_element_is_self_closing(
    lol_html_element_t *element
);

// Whether the element can have inner content.  Returns `true` unless the element is an [HTML void
// element](https://html.spec.whatwg.org/multipage/syntax.html#void-elements) or has a
// self-closing tag (eg, `<foo />`).
bool lol_html_element_can_have_content(
    lol_html_element_t *element
);

// Returns the namespace URI of the element.
//
// NOTE: This method returns static zero-terminated C string, so it don't
// need to be freed.
const char* lol_html_element_namespace_uri_get(const lol_html_element_t *element);

// Returns the iterator over the element attributes.
//
// WARNING: The iterator is valid only during the handler execution and
// should never be leaked outside of it.
//
// Use `lol_html_attributes_iterator_free` function to deallocate
// returned iterator.
lol_html_attributes_iterator_t *lol_html_attributes_iterator_get(
    const lol_html_element_t *element
);

// Frees the memory held by the attribute iterator.
void lol_html_attributes_iterator_free(lol_html_attributes_iterator_t *iterator);

// Advances the iterator and returns next attribute.
//
// Returns NULL if iterator has been exhausted.
//
// WARNING: Returned attribute is valid only during the handler
// execution and should never be leaked outside of it.
const lol_html_attribute_t *lol_html_attributes_iterator_next(
    lol_html_attributes_iterator_t *iterator
);

// Returns the attribute name.
lol_html_str_t lol_html_attribute_name_get(const lol_html_attribute_t *attribute);

// Returns the attribute name, preserving its case.
lol_html_str_t lol_html_attribute_name_get_preserve_case(const lol_html_attribute_t *attribute);

// Returns the attribute value.
lol_html_str_t lol_html_attribute_value_get(const lol_html_attribute_t *attribute);

// Returns [`SourceLocationBytes`].
//
// `element` must be valid and non-`NULL`.
lol_html_source_location_bytes_t lol_html_element_source_location_bytes(lol_html_element_t *element);

// Returns the attribute value. The `data` field will be NULL if an attribute with the given name
// doesn't exist on the element.
//
// Name should be a valid UTF8-string.
//
// If the provided name is invalid UTF8-string the function returns NULL as well.
// Therefore one should always check `lol_html_take_last_error` result after the call.
lol_html_str_t lol_html_element_get_attribute(
    const lol_html_element_t *element,
    const char *name,
    size_t name_len
);

// Returns 1 if element has attribute with the given name, and 0 otherwise.
// Returns -1 in case of an error.
//
// Name should be a valid UTF8-string.
int lol_html_element_has_attribute(
    const lol_html_element_t *element,
    const char *name,
    size_t name_len
);

// Updates the attribute value if attribute with the given name already exists on
// the element, or creates adds new attribute with given name and value otherwise.
//
// Name and value should be valid UTF8-strings.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_element_set_attribute(
    lol_html_element_t *element,
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
// can be obtained using `lol_html_take_last_error` function.
int lol_html_element_remove_attribute(
    lol_html_element_t *element,
    const char *name,
    size_t name_len
);

// Inserts the content string before the element either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
//
// Calls [`Element::before`].
int lol_html_element_before(
    lol_html_element_t *element,
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
// can be obtained using `lol_html_take_last_error` function.
int lol_html_element_prepend(
    lol_html_element_t *element,
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
// can be obtained using `lol_html_take_last_error` function.
//
// Calls [`Element::append`].
int lol_html_element_append(
    lol_html_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string right after the element's end tag as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_element_after(
    lol_html_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Sets either text or HTML inner content of the element.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_element_set_inner_content(
    lol_html_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Replaces the element with the provided text or HTML content.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_element_replace(
    lol_html_element_t *element,
    const char *content,
    size_t content_len,
    bool is_html
);

// Removes the element.
void lol_html_element_remove(const lol_html_element_t *element);

// Removes the element, but leaves its inner content intact.
void lol_html_element_remove_and_keep_content(const lol_html_element_t *element);

// Returns `true` if the element has been removed.
bool lol_html_element_is_removed(const lol_html_element_t *element);

// Attaches custom user data to the element.
//
// The same element can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some processing
// state on the element, so it can be shared between handlers.
void lol_html_element_user_data_set(
    const lol_html_element_t *element,
    void *user_data
);

// Returns user data attached to the element.
void *lol_html_element_user_data_get(const lol_html_element_t *element);

// Adds content handlers to the builder for the end tag of the given element.
//
// Subsequent calls to the method on the same element adds new handler.
// They will run in the order in which they were registered.
//
// The handler can optionally have associated user data which will be
// passed to the handler on each invocation along with the rewritable
// unit argument.
//
// If the handler returns LOL_HTML_STOP directive then rewriting
// stops immediately and `write()` or `end()` of the rewriter methods
// return an error code.
//
// Not all elements (for example, `<br>`) support end tags. If this function is
// called on such an element, this function returns an error code as described
// below.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
//
// WARNING: Pointers passed to handlers are valid only during the
// handler execution. So they should never be leaked outside of handlers.
int lol_html_element_add_end_tag_handler(
    lol_html_element_t* element,
    lol_html_end_tag_handler_t end_tag_handler,
    void* user_data
);

// Clears the handlers that would run on the end tag of the given element.
void lol_html_element_clear_end_tag_handlers(lol_html_element_t* element);

// Inserts the content string before the element's end tag either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
//
// Calls [`EndTag::before`].
int lol_html_end_tag_before(
    lol_html_end_tag_t *end_tag,
    const char *content,
    size_t content_len,
    bool is_html
);

// Inserts the content string right after the element's end tag as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_end_tag_after(
    lol_html_end_tag_t *end_tag,
    const char *content,
    size_t content_len,
    bool is_html
);

// Removes the end tag.
// `end_tag` must be valid and non-`NULL`.
//
// Calls [`EndTag::remove`].
void lol_html_end_tag_remove(lol_html_end_tag_t *end_tag);

// Returns the end tag name.
lol_html_str_t lol_html_end_tag_name_get(const lol_html_end_tag_t *end_tag);

// Returns the end tag name, preserving its case.
lol_html_str_t lol_html_end_tag_name_get_preserve_case(const lol_html_end_tag_t *end_tag);

// Sets the tag name of the end tag.
//
// Name should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_end_tag_name_set(
    lol_html_end_tag_t *end_tag,
    const char *name,
    size_t name_len
);

// Inserts the content at the end of the document, either as raw text or as HTML.
//
// The content should be a valid UTF-8 string.
//
// Returns 0 if successful, and -1 otherwise. The actual error message
// can be obtained using the `lol_html_take_last_error` function.
int lol_html_doc_end_append(
    lol_html_doc_end_t *doc_end,
    const char *content,
    size_t content_len,
    bool is_html
);



// [`Element::streaming_prepend`]
//
// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `element` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
int lol_html_element_streaming_prepend(lol_html_element_t *element,
                                       lol_html_streaming_handler_t *streaming_writer);

// [`Element::streaming_append`]
//
// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `element` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
int lol_html_element_streaming_append(lol_html_element_t *element,
                                      lol_html_streaming_handler_t *streaming_writer);

// [`Element::streaming_before`]
//
// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `element` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
int lol_html_element_streaming_before(lol_html_element_t *element,
                                      lol_html_streaming_handler_t *streaming_writer);

// [`Element::streaming_after`]
//
// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `element` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
int lol_html_element_streaming_after(lol_html_element_t *element,
                                     lol_html_streaming_handler_t *streaming_writer);

// [`Element::streaming_set_inner_content`]
//
// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `element` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
int lol_html_element_streaming_set_inner_content(lol_html_element_t *element,
                                                 lol_html_streaming_handler_t *streaming_writer);

// [`Element::streaming_replace`]
//
// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `element` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
int lol_html_element_streaming_replace(lol_html_element_t *element,
                                       lol_html_streaming_handler_t *streaming_writer);

// Returns [`SourceLocationBytes`].
//
// `end_tag` must be valid and non-`NULL`.
lol_html_source_location_bytes_t lol_html_end_tag_source_location_bytes(lol_html_end_tag_t *end_tag);

// [`EndTag::streaming_before`]
//
// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `end_tag` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
//
// Calls [`EndTag::streaming_before`].
int lol_html_end_tag_streaming_before(lol_html_end_tag_t *end_tag,
                                      lol_html_streaming_handler_t *streaming_writer);

// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `end_tag` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
//
// Calls [`EndTag::streaming_after`].
int lol_html_end_tag_streaming_after(lol_html_end_tag_t *end_tag,
                                     lol_html_streaming_handler_t *streaming_writer);

// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `end_tag` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
//
// Calls [`EndTag::streaming_replace`].
int lol_html_end_tag_streaming_replace(lol_html_end_tag_t *end_tag,
                                       lol_html_streaming_handler_t *streaming_writer);


// Write another piece of UTF-8 data to the output. Returns `0` on success, and `-1` if it wasn't valid UTF-8.
// All pointers must be non-NULL.
int lol_html_streaming_sink_write_str(lol_html_streaming_sink_t *sink,
                                         const char *string_utf8,
                                         size_t string_utf8_len,
                                         bool is_html);

// [`StreamingHandlerSink::write_utf8_chunk`]
//
// Writes as much of the given UTF-8 fragment as possible, converting the encoding and HTML-escaping if `is_html` is `false`.
//
// The `bytes_utf8` doesn't need to be a complete UTF-8 string, as long as consecutive calls to this function create a valid UTF-8 string.
// Any incomplete UTF-8 sequence at the end of the content is buffered and flushed as soon as it's completed.
//
// Other functions like [`lol_html_streaming_sink_write_str`] should not be called after a
// `lol_html_streaming_sink_write_utf8_chunk` call with an incomplete UTF-8 sequence.
//
// Returns `0` on success, and `-1` if it wasn't valid UTF-8.
// All pointers must be non-`NULL`.
int lol_html_streaming_sink_write_utf8_chunk(lol_html_streaming_sink_t *sink,
                                                const char *bytes_utf8,
                                                size_t bytes_utf8_len,
                                                bool is_html);

// Text chunk
//---------------------------------------------------------------------

// Returns a fat pointer to the UTF8 representation of content of the chunk.
//
// If the chunk is last in the current text node then content can be an empty string.
//
// WARNING: The pointer is valid only during the handler execution and
// should never be leaked outside of handlers.
lol_html_text_chunk_content_t lol_html_text_chunk_content_get(
    const lol_html_text_chunk_t *chunk
);

// Inserts the content string before the text chunk either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_text_chunk_before(
    lol_html_text_chunk_t *chunk,
    const char *content,
    size_t content_len,
    bool is_html
);


// Inserts the content string after the text chunk either as raw text or as HTML.
//
// Content should be a valid UTF8-string.
//
// Returns 0 in case of success and -1 otherwise. The actual error message
// can be obtained using `lol_html_take_last_error` function.
int lol_html_text_chunk_after(
    lol_html_text_chunk_t *chunk,
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
// can be obtained using `lol_html_take_last_error` function.
int lol_html_text_chunk_replace(
    lol_html_text_chunk_t *chunk,
    const char *content,
    size_t content_len,
    bool is_html
);

// Removes the text chunk.
void lol_html_text_chunk_remove(lol_html_text_chunk_t *chunk);

// Returns `true` if the text chunk has been removed.
bool lol_html_text_chunk_is_removed(const lol_html_text_chunk_t *chunk);

// Returns `true` if the chunk is last in the current text node.
// `text_chunk` must be valid and non-`NULL`.
// Returns `_Bool`.
//
// Calls [`TextChunk::last_in_text_node`].
bool lol_html_text_chunk_is_last_in_text_node(lol_html_text_chunk_t *text_chunk);

// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `text_chunk` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
//
// Calls [`TextChunk::streaming_before`].
int lol_html_text_chunk_streaming_before(lol_html_text_chunk_t *text_chunk,
        lol_html_streaming_handler_t *streaming_writer);

// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `text_chunk` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
//
// Calls [`TextChunk::streaming_after`].
int lol_html_text_chunk_streaming_after(lol_html_text_chunk_t *text_chunk,
                                        lol_html_streaming_handler_t *streaming_writer);

// The [`CStreamingHandler`] contains callbacks that will be called
// when the content needs to be written.
//
// `streaming_writer` is copied immediately, and doesn't have a stable address.
// `streaming_writer` may be used from another thread (`Send`), but it's only going
// to be used by one thread at a time (`!Sync`).
//
// `text_chunk` must be valid and non-`NULL`.
// If `streaming_writer` is `NULL`, an error will be reported.
//
// Returns 0 on success.
//
// Calls [`TextChunk::streaming_replace`].
int lol_html_text_chunk_streaming_replace(lol_html_text_chunk_t *text_chunk,
        lol_html_streaming_handler_t *streaming_writer);

// Returns [`SourceLocationBytes`].
//
// `text_chunk` must be valid and non-`NULL`.
lol_html_source_location_bytes_t lol_html_text_chunk_source_location_bytes(lol_html_text_chunk_t *text_chunk);

// Attaches custom user data to the text chunk.
//
// The same text chunk can be passed to multiple handlers if it has been
// captured by multiple selectors. It might be handy to store some processing
// state on the chunk, so it can be shared between handlers.
void lol_html_text_chunk_user_data_set(lol_html_text_chunk_t *chunk, void *user_data);

// Returns user data attached to the text chunk.
void *lol_html_text_chunk_user_data_get(const lol_html_text_chunk_t *chunk);

#if defined(__cplusplus)
}  // extern C
#endif

#endif // LOL_HTML_H
