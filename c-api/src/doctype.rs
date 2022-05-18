use super::*;

#[no_mangle]
pub extern "C" fn lol_html_doctype_name_get(doctype: *const Doctype) -> Str {
    Str::from_opt(to_ref!(doctype).name())
}

#[no_mangle]
pub extern "C" fn lol_html_doctype_public_id_get(doctype: *const Doctype) -> Str {
    Str::from_opt(to_ref!(doctype).public_id())
}

#[no_mangle]
pub extern "C" fn lol_html_doctype_system_id_get(doctype: *const Doctype) -> Str {
    Str::from_opt(to_ref!(doctype).system_id())
}

#[no_mangle]
pub extern "C" fn lol_html_doctype_user_data_set(doctype: *mut Doctype, user_data: *mut c_void) {
    to_ref_mut!(doctype).set_user_data(user_data);
}

#[no_mangle]
pub extern "C" fn lol_html_doctype_user_data_get(doctype: *const Doctype) -> *mut c_void {
    get_user_data!(doctype)
}

#[no_mangle]
pub extern "C" fn lol_html_doctype_remove(doctype: *mut Doctype) {
    to_ref_mut!(doctype).remove();
}

#[no_mangle]
pub extern "C" fn lol_html_doctype_is_removed(doctype: *const Doctype) -> bool {
    to_ref!(doctype).removed()
}
