use super::*;
use std::slice::Iter;

#[no_mangle]
pub extern "C" fn lol_html_element_tag_name_get(element: *const Element) -> Str {
    let element = to_ref!(element);

    Str::new(element.tag_name())
}

#[no_mangle]
pub extern "C" fn lol_html_element_tag_name_get_preserve_case(element: *const Element) -> Str {
    let element = to_ref!(element);

    Str::new(element.tag_name_preserve_case())
}

#[no_mangle]
pub extern "C" fn lol_html_element_tag_name_set(
    element: *mut Element,
    name: *const c_char,
    name_len: size_t,
) -> c_int {
    let element = to_ref_mut!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };

    unwrap_or_ret_err_code! { element.set_tag_name(name) };

    0
}

#[no_mangle]
pub extern "C" fn lol_html_element_is_self_closing(element: *mut Element) -> bool {
    to_ref!(element).is_self_closing()
}

#[no_mangle]
pub extern "C" fn lol_html_element_can_have_content(element: *mut Element) -> bool {
    to_ref!(element).can_have_content()
}

#[no_mangle]
pub extern "C" fn lol_html_element_namespace_uri_get(element: *mut Element) -> *const c_char {
    let element = to_ref!(element);

    match element.namespace_uri() {
        "http://www.w3.org/1999/xhtml" => static_c_str!("http://www.w3.org/1999/xhtml"),
        "http://www.w3.org/2000/svg" => static_c_str!("http://www.w3.org/2000/svg"),
        "http://www.w3.org/1998/Math/MathML" => static_c_str!("http://www.w3.org/1998/Math/MathML"),
        _ => unreachable!("Unknown namespace URI"),
    }
}

#[no_mangle]
pub extern "C" fn lol_html_attributes_iterator_get<'r, 't>(
    element: *const Element<'r, 't>,
) -> *mut Iter<'r, Attribute<'t>> {
    let attributes = to_ref!(element).attributes();

    to_ptr_mut(attributes.iter())
}

#[no_mangle]
pub extern "C" fn lol_html_attributes_iterator_next<'r, 't>(
    iterator: *mut Iter<'r, Attribute<'t>>,
) -> *const Attribute<'t> {
    let iterator = to_ref_mut!(iterator);

    match iterator.next() {
        Some(attr) => attr,
        None => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn lol_html_attributes_iterator_free(iterator: *mut Iter<Attribute>) {
    drop(to_box!(iterator));
}

#[no_mangle]
pub extern "C" fn lol_html_attribute_name_get(attribute: *const Attribute) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.name())
}

#[no_mangle]
pub extern "C" fn lol_html_attribute_name_get_preserve_case(attribute: *const Attribute) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.name_preserve_case())
}

#[no_mangle]
pub extern "C" fn lol_html_attribute_value_get(attribute: *const Attribute) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.value())
}

#[no_mangle]
pub extern "C" fn lol_html_element_get_attribute(
    element: *const Element,
    name: *const c_char,
    name_len: size_t,
) -> Str {
    let element = to_ref!(element);
    let name = unwrap_or_ret!(to_str!(name, name_len), Str::from_opt(None));

    Str::from_opt(element.get_attribute(name))
}

#[no_mangle]
pub extern "C" fn lol_html_element_has_attribute(
    element: *const Element,
    name: *const c_char,
    name_len: size_t,
) -> c_int {
    let element = to_ref!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };

    if element.has_attribute(name) {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn lol_html_element_set_attribute(
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

#[no_mangle]
pub extern "C" fn lol_html_element_remove_attribute(
    element: *mut Element,
    name: *const c_char,
    name_len: size_t,
) -> c_int {
    let element = to_ref_mut!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };

    element.remove_attribute(name);

    0
}

#[no_mangle]
pub extern "C" fn lol_html_element_before(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.before(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_element_prepend(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.prepend(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_element_append(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.append(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_element_after(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.after(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_element_set_inner_content(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.set_inner_content(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_element_replace(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.replace(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_element_remove(element: *mut Element) {
    to_ref_mut!(element).remove();
}

#[no_mangle]
pub extern "C" fn lol_html_element_remove_and_keep_content(element: *mut Element) {
    to_ref_mut!(element).remove_and_keep_content();
}

#[no_mangle]
pub extern "C" fn lol_html_element_is_removed(element: *mut Element) -> bool {
    to_ref_mut!(element).removed()
}

#[no_mangle]
pub extern "C" fn lol_html_element_user_data_set(element: *mut Element, user_data: *mut c_void) {
    to_ref_mut!(element).set_user_data(user_data);
}

#[no_mangle]
pub extern "C" fn lol_html_element_user_data_get(element: *mut Element) -> *mut c_void {
    get_user_data!(element)
}

type EndTagHandler = unsafe extern "C" fn(*mut EndTag, *mut c_void) -> RewriterDirective;

#[no_mangle]
pub extern "C" fn lol_html_element_add_end_tag_handler(
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

#[no_mangle]
pub extern "C" fn lol_html_element_clear_end_tag_handlers(element: *mut Element) {
    let element = to_ref_mut!(element);
    if let Some(handlers) = element.end_tag_handlers() {
        handlers.clear();
    }
}

#[no_mangle]
pub extern "C" fn lol_html_end_tag_before(
    end_tag: *mut EndTag,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { end_tag.before(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_end_tag_after(
    end_tag: *mut EndTag,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { end_tag.after(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn lol_html_end_tag_remove(end_tag: *mut EndTag) {
    to_ref_mut!(end_tag).remove();
}

#[no_mangle]
pub extern "C" fn lol_html_end_tag_name_get(end_tag: *mut EndTag) -> Str {
    let tag = to_ref_mut!(end_tag);
    Str::new(tag.name())
}

#[no_mangle]
pub extern "C" fn lol_html_end_tag_name_get_preserve_case(end_tag: *mut EndTag) -> Str {
    let tag = to_ref_mut!(end_tag);
    Str::new(tag.name_preserve_case())
}

#[no_mangle]
pub extern "C" fn lol_html_end_tag_name_set(
    end_tag: *mut EndTag,
    name: *const c_char,
    len: size_t,
) -> c_int {
    let tag = to_ref_mut!(end_tag);
    let name = unwrap_or_ret_err_code! { to_str!(name, len) };
    tag.set_name_str(name.to_string());
    0
}
