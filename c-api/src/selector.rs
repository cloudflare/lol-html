use super::*;

#[no_mangle]
pub extern "C" fn cool_thing_selector_parse(
    selector: *const c_char,
    selector_len: size_t,
) -> *mut Selector {
    let selector = unwrap_or_ret_null! { to_str!(selector, selector_len) };
    let selector = unwrap_or_ret_null! { selector.parse::<Selector>() };

    to_ptr_mut(selector)
}

#[no_mangle]
pub extern "C" fn cool_thing_selector_free(selector: *mut Selector) {
    drop(to_box!(selector));
}
