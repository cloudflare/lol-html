use super::*;

#[no_mangle]
pub extern "C" fn cool_thing_doctype_name_get(doctype: *const Doctype) -> *const Str {
    Str::opt_ptr(to_ref!(doctype).name())
}

#[no_mangle]
pub extern "C" fn cool_thing_doctype_public_id_get(doctype: *const Doctype) -> *const Str {
    Str::opt_ptr(to_ref!(doctype).public_id())
}

#[no_mangle]
pub extern "C" fn cool_thing_doctype_system_id_get(doctype: *const Doctype) -> *const Str {
    Str::opt_ptr(to_ref!(doctype).system_id())
}

#[no_mangle]
pub extern "C" fn cool_thing_doctype_user_data_set(doctype: *mut Doctype, user_data: *mut c_void) {
    to_ref_mut!(doctype).set_user_data(user_data);
}

#[no_mangle]
pub extern "C" fn cool_thing_doctype_user_data_get(doctype: *const Doctype) -> *mut c_void {
    get_user_data!(doctype)
}
