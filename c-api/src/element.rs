use super::*;
use std::slice::Iter;

#[no_mangle]
pub extern "C" fn cool_thing_element_tag_name_get(element: *const Element<'_, '_>) -> Str {
    let element = to_ref!(element);

    Str::new(element.tag_name())
}

#[no_mangle]
pub extern "C" fn cool_thing_element_tag_name_set(
    element: *mut Element<'_, '_>,
    name: *const c_char,
    name_len: size_t,
) -> c_int {
    let element = to_ref_mut!(element);
    let name = unwrap_or_ret_err_code! { to_str!(name, name_len) };

    unwrap_or_ret_err_code! { element.set_tag_name(name) };

    0
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
pub extern "C" fn cool_thing_attributes_iterator_free(iterator: *mut Iter<Attribute<'_>>) {
    drop(to_box!(iterator));
}

#[no_mangle]
pub extern "C" fn cool_thing_attribute_name_get(attribute: *const Attribute<'_>) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.name())
}

#[no_mangle]
pub extern "C" fn cool_thing_attribute_value_get(attribute: *const Attribute<'_>) -> Str {
    let attribute = to_ref!(attribute);

    Str::new(attribute.value())
}

#[no_mangle]
pub extern "C" fn cool_thing_element_get_attribute(
    element: *const Element<'_, '_>,
    name: *const c_char,
    name_len: size_t,
) -> *const Str {
    let element = to_ref!(element);
    let name = unwrap_or_ret_null! { to_str!(name, name_len) };

    Str::opt_ptr(element.get_attribute(name))
}

#[no_mangle]
pub extern "C" fn cool_thing_element_has_attribute(
    element: *const Element<'_, '_>,
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
    element: *mut Element<'_, '_>,
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
    element: *mut Element<'_, '_>,
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
    element: *mut Element<'_, '_>,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.before(content, content_len, is_html) }
}

#[no_mangle]
pub extern "C" fn cool_thing_element_prepend(
    element: *mut Element<'_, '_>,
    content: *const c_char,
    content_len: size_t,
    is_html: bool,
) -> c_int {
    content_insertion_fn_body! { element.prepend(content, content_len, is_html) }
}

// TODO
// set_inner_content
// after
// append
// replace
// remove
// remove_and_keep_content
// removed
