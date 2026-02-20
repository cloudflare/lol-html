use super::*;
use std::slice::Iter;

/// Returns the tag name of the element.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_tag_name_get(element: *const Element) -> Str {
    let element = to_ref!(element);

    Str::new(element.tag_name())
}

/// Returns the tag name of the element, preserving its case.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_tag_name_get_preserve_case(
    element: *const Element,
) -> Str {
    let element = to_ref!(element);

    Str::new(element.tag_name_preserve_case())
}

/// Sets the tag name of the element.
///
/// Name should be a valid UTF8-string.
///
/// Returns 0 in case of success and -1 otherwise. The actual error message
/// can be obtained using `lol_html_take_last_error` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_tag_name_set(
    element: *mut Element,
    name: *const c_char,
    name_len: size_t,
) -> c_int {
    let element = to_ref_mut!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };

    unwrap_or_ret_err_code! { element.set_tag_name(name) };

    0
}

/// Returns the namespace URI of the element.
///
/// NOTE: This method returns static zero-terminated C string, so it don't
/// need to be freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_namespace_uri_get(
    element: *mut Element,
) -> *const c_char {
    let element = to_ref!(element);

    element.namespace_uri_c_str().as_ptr()
}

/// Returns the iterator over the element attributes.
///
/// WARNING: The iterator is valid only during the handler execution and
/// should never be leaked outside of it.
///
/// Use `lol_html_attributes_iterator_free` function to deallocate
/// returned iterator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_attributes_iterator_get<'r, 't>(
    element: *const Element<'r, 't>,
) -> *mut Iter<'r, Attribute<'t>> {
    let attributes = to_ref!(element).attributes();

    to_ptr_mut(attributes.iter())
}

// Advances the iterator and returns next attribute.
//
// Returns NULL if iterator has been exhausted.
//
// WARNING: Returned attribute is valid only during the handler
// execution and should never be leaked outside of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_attributes_iterator_next<'t>(
    iterator: *mut Iter<'_, Attribute<'t>>,
) -> *const Attribute<'t> {
    let iterator = to_ref_mut!(iterator);

    match iterator.next() {
        Some(attr) => attr,
        None => ptr::null(),
    }
}

// Frees the memory held by the attribute iterator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_attributes_iterator_free(iterator: *mut Iter<Attribute>) {
    drop(to_box!(iterator));
}

/// Returns the attribute name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_attribute_name_get(attribute: *const Attribute) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.name())
}

/// Returns the attribute name, preserving its case.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_attribute_name_get_preserve_case(
    attribute: *const Attribute,
) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.name_preserve_case())
}

/// Returns the attribute value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_attribute_value_get(attribute: *const Attribute) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.value())
}

/// Returns the attribute value. The `data` field will be NULL if an attribute with the given name
/// doesn't exist on the element.
///
/// Name should be a valid UTF8-string.
///
/// If the provided name is invalid UTF8-string the function returns NULL as well.
/// Therefore one should always check `lol_html_take_last_error` result after the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_get_attribute(
    element: *const Element,
    name: *const c_char,
    name_len: size_t,
) -> Str {
    let element = to_ref!(element);
    let name = unwrap_or_ret!(to_str!(name, name_len), Str::EMPTY);

    Str::from_opt(element.get_attribute(name))
}

/// Returns 1 if element has attribute with the given name, and 0 otherwise.
/// Returns -1 in case of an error.
///
/// Name should be a valid UTF8-string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_has_attribute(
    element: *const Element,
    name: *const c_char,
    name_len: size_t,
) -> c_int {
    let element = to_ref!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };

    if element.has_attribute(name) { 1 } else { 0 }
}

/// Updates the attribute value if attribute with the given name already exists on
/// the element, or creates adds new attribute with given name and value otherwise.
///
/// Name and value should be valid UTF8-strings.
///
/// Returns 0 in case of success and -1 otherwise. The actual error message
/// can be obtained using `lol_html_take_last_error` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_set_attribute(
    element: *mut Element,
    name: *const c_char,
    name_len: size_t,
    value: *const c_char,
    value_len: size_t,
) -> c_int {
    let element = to_ref_mut!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };
    let value = unwrap_or_ret_err_code! { to_str!(value, value_len) };

    unwrap_or_ret_err_code! { element.set_attribute(name, value) };

    0
}

/// Removes the attribute with the given name from the element.
///
/// Name should be a valid UTF8-string.
///
/// Returns 0 in case of success and -1 otherwise. The actual error message
/// can be obtained using `lol_html_take_last_error` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_remove_attribute(
    element: *mut Element,
    name: *const c_char,
    name_len: size_t,
) -> c_int {
    let element = to_ref_mut!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };

    element.remove_attribute(name);

    0
}

impl_content_mutation_handlers! { element: Element [
    /// Inserts the content string right after the element's start tag
    /// either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_element_prepend => prepend,
    /// Inserts the content string right before the element's end tag
    /// either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_element_append => append,
    /// Inserts the content string before the element either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_element_before => before,
    /// Inserts the content string right after the element's end tag as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_element_after => after,
    /// Sets either text or HTML inner content of the element.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_element_set_inner_content => set_inner_content,
    /// Replaces the element with the provided text or HTML content.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_element_replace => replace,
    /// Removes the element.
    @VOID lol_html_element_remove => remove,
    /// Removes the element, but leaves its inner content intact.
    @VOID lol_html_element_remove_and_keep_content => remove_and_keep_content,
    /// Returns `true` if the element has been removed.
    @BOOL lol_html_element_is_removed => removed,
    /// Whether the tag syntactically ends with `/>`. In HTML content this is purely a decorative, unnecessary, and has no effect of any kind.
    ///
    /// The `/>` syntax only affects parsing of elements in foreign content (SVG and MathML).
    /// It will never close any HTML tags that aren't already defined as void in HTML.
    ///
    /// This function only reports the parsed syntax, and will not report which elements are actually void in HTML.
    /// Use `lol_html_element_can_have_content` to check if the element is non-void.
    ///
    /// If the `/` is part of an unquoted attribute, it's not parsed as the self-closing syntax.
    @BOOL lol_html_element_is_self_closing => is_self_closing,
    /// Whether the element can have inner content.  Returns `true` unless the element is an [HTML void
    /// element](https://html.spec.whatwg.org/multipage/syntax.html#void-elements) or has a
    /// self-closing tag (eg, `<foo />`).
    @BOOL lol_html_element_can_have_content => can_have_content,
    @STREAM lol_html_element_streaming_prepend => streaming_prepend,
    @STREAM lol_html_element_streaming_append => streaming_append,
    @STREAM lol_html_element_streaming_before => streaming_before,
    @STREAM lol_html_element_streaming_after => streaming_after,
    @STREAM lol_html_element_streaming_set_inner_content => streaming_set_inner_content,
    @STREAM lol_html_element_streaming_replace => streaming_replace,
    lol_html_element_source_location_bytes => source_location_bytes,
] }

/// Attaches custom user data to the element.
///
/// The same element can be passed to multiple handlers if it has been
/// captured by multiple selectors. It might be handy to store some processing
/// state on the element, so it can be shared between handlers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_user_data_set(
    element: *mut Element,
    user_data: *mut c_void,
) {
    to_ref_mut!(element).set_user_data(user_data);
}

/// Returns user data attached to the element.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_user_data_get(element: *mut Element) -> *mut c_void {
    get_user_data!(element)
}

type EndTagHandler = unsafe extern "C" fn(*mut EndTag, *mut c_void) -> RewriterDirective;

/// Adds content handlers to the builder for the end tag of the given element.
///
/// Subsequent calls to the method on the same element adds new handler.
/// They will run in the order in which they were registered.
///
/// The handler can optionally have associated user data which will be
/// passed to the handler on each invocation along with the rewritable
/// unit argument.
///
/// If the handler returns LOL_HTML_STOP directive then rewriting
/// stops immediately and `write()` or `end()` of the rewriter methods
/// return an error code.
///
/// Not all elements (for example, `<br>`) support end tags. If this function is
/// called on such an element, this function returns an error code as described
/// below.
///
/// Returns 0 in case of success and -1 otherwise. The actual error message
/// can be obtained using `lol_html_take_last_error` function.
///
/// WARNING: Pointers passed to handlers are valid only during the
/// handler execution. So they should never be leaked outside of handlers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_add_end_tag_handler(
    element: *mut Element,
    handler: EndTagHandler,
    user_data: *mut c_void,
) -> c_int {
    let element = to_ref_mut!(element);

    let handlers = unwrap_or_ret_err_code! {
        element.end_tag_handlers().ok_or("No end tag.")
    };

    handlers.push(Box::new(move |end_tag| {
        match unsafe { handler(end_tag, user_data) } {
            RewriterDirective::Continue => Ok(()),
            RewriterDirective::Stop => Err("The rewriter has been stopped.".into()),
        }
    }));

    0
}

/// Clears the handlers that would run on the end tag of the given element.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_element_clear_end_tag_handlers(element: *mut Element) {
    let element = to_ref_mut!(element);
    if let Some(handlers) = element.end_tag_handlers() {
        handlers.clear();
    }
}

impl_content_mutation_handlers! { end_tag: EndTag [
    /// Inserts the content string before the element's end tag either as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_end_tag_before => before,
    /// Inserts the content string right after the element's end tag as raw text or as HTML.
    ///
    /// Content should be a valid UTF8-string.
    ///
    /// Returns 0 in case of success and -1 otherwise. The actual error message
    /// can be obtained using `lol_html_take_last_error` function.
    lol_html_end_tag_after => after,
    lol_html_end_tag_replace => replace,
    /// Removes the end tag.
    @VOID lol_html_end_tag_remove => remove,
    @STREAM lol_html_end_tag_streaming_before => streaming_before,
    @STREAM lol_html_end_tag_streaming_after => streaming_after,
    @STREAM lol_html_end_tag_streaming_replace => streaming_replace,
    lol_html_end_tag_source_location_bytes => source_location_bytes,
] }

/// Returns the end tag name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_end_tag_name_get(end_tag: *mut EndTag) -> Str {
    let tag = to_ref_mut!(end_tag);
    Str::new(tag.name())
}

/// Returns the end tag name, preserving its case.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_end_tag_name_get_preserve_case(end_tag: *mut EndTag) -> Str {
    let tag = to_ref_mut!(end_tag);
    Str::new(tag.name_preserve_case())
}

/// Sets the tag name of the end tag.
///
/// Name should be a valid UTF8-string.
///
/// Returns 0 in case of success and -1 otherwise. The actual error message
/// can be obtained using `lol_html_take_last_error` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lol_html_end_tag_name_set(
    end_tag: *mut EndTag,
    name: *const c_char,
    len: size_t,
) -> c_int {
    let tag = to_ref_mut!(end_tag);
    let name = unwrap_or_ret_err_code! { to_str!(name, len) };
    tag.set_name_str(name.to_string());
    0
}
