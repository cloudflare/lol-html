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
