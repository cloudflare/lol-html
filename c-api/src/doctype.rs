use super::*;

#[no_mangle]
pub unsafe extern "C" fn lol_html_doctype_name_get(doctype: *const Doctype) -> Str {
    Str::from_opt(to_ref!(doctype).name())
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_doctype_public_id_get(doctype: *const Doctype) -> Str {
    Str::from_opt(to_ref!(doctype).public_id())
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_doctype_system_id_get(doctype: *const Doctype) -> Str {
    Str::from_opt(to_ref!(doctype).system_id())
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_doctype_user_data_set(
    doctype: *mut Doctype,
    user_data: *mut c_void,
) {
    to_ref_mut!(doctype).set_user_data(user_data);
}

#[no_mangle]
pub unsafe extern "C" fn lol_html_doctype_user_data_get(doctype: *const Doctype) -> *mut c_void {
    get_user_data!(doctype)
}

impl_content_mutation_handlers! { doctype: Doctype [
    /// Removes the doctype.
    @VOID lol_html_doctype_remove => remove,
    /// Returns `true` if the doctype has been removed.
    @BOOL lol_html_doctype_is_removed => removed,
    lol_html_doctype_source_location_bytes => source_location_bytes,
] }
