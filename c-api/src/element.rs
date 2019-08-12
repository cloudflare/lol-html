use super::*;
use std::slice::Iter;

#[no_mangle]
pub extern "C" fn cool_thing_element_tag_name_get(element: *const Element) -> Str {
    let element = to_ref!(element);

    Str::new(element.tag_name())
}

#[no_mangle]
pub extern "C" fn cool_thing_element_tag_name_set(
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
pub extern "C" fn cool_thing_element_namespace_uri_get(element: *mut Element) -> *const c_char {
    let element = to_ref!(element);

    match element.namespace_uri() {
        "http://www.w3.org/1999/xhtml" => static_c_str!("http://www.w3.org/1999/xhtml"),
        "http://www.w3.org/2000/svg" => static_c_str!("http://www.w3.org/2000/svg"),
        "http://www.w3.org/1998/Math/MathML" => static_c_str!("http://www.w3.org/1998/Math/MathML"),
        _ => unreachable!("Unknown namespace URI"),
    }
}

#[no_mangle]
pub extern "C" fn cool_thing_attributes_iterator_get<'r, 't>(
    element: *const Element<'r, 't>,
) -> *mut Iter<'r, Attribute<'t>> {
    let attributes = to_ref!(element).attributes();

    to_ptr_mut(attributes.iter())
}

#[no_mangle]
pub extern "C" fn cool_thing_attributes_iterator_next<'r, 't>(
    iterator: *mut Iter<'r, Attribute<'t>>,
) -> *const Attribute<'t> {
    let iterator = to_ref_mut!(iterator);

    match iterator.next() {
        Some(attr) => attr,
        None => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn cool_thing_attributes_iterator_free(iterator: *mut Iter<Attribute>) {
    drop(to_box!(iterator));
}

#[no_mangle]
pub extern "C" fn cool_thing_attribute_name_get(attribute: *const Attribute) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.name())
}

#[no_mangle]
pub extern "C" fn cool_thing_attribute_value_get(attribute: *const Attribute) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.value())
}

#[no_mangle]
pub extern "C" fn cool_thing_element_get_attribute(
    element: *const Element,
    name: *const c_char,
    name_len: size_t,
) -> *const Str {
    let element = to_ref!(element);
    let name = unwrap_or_ret_null! { to_str!(name, name_len) };

    Str::opt_ptr(element.get_attribute(name))
}

#[no_mangle]
pub extern "C" fn cool_thing_element_has_attribute(
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
pub extern "C" fn cool_thing_element_set_attribute(
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
pub extern "C" fn cool_thing_element_remove_attribute(
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
pub extern "C" fn cool_thing_element_before(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.before(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_element_prepend(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.prepend(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_element_append(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.append(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_element_after(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.after(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_element_set_inner_content(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.set_inner_content(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_element_replace(
    element: *mut Element,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.replace(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_element_remove(element: *mut Element) {
    to_ref_mut!(element).remove();
}

#[no_mangle]
pub extern "C" fn cool_thing_element_remove_and_keep_content(element: *mut Element) {
    to_ref_mut!(element).remove_and_keep_content();
}

#[no_mangle]
pub extern "C" fn cool_thing_element_is_removed(element: *mut Element) -> bool {
    to_ref_mut!(element).removed()
}

#[no_mangle]
pub extern "C" fn cool_thing_element_user_data_set(element: *mut Element, user_data: *mut c_void) {
    to_ref_mut!(element).set_user_data(user_data);
}

#[no_mangle]
pub extern "C" fn cool_thing_element_user_data_get(element: *mut Element) -> *mut c_void {
    get_user_data!(element)
}
