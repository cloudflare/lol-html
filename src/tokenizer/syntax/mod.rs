#[macro_use]
mod cdata_section;

#[macro_use]
mod data;

#[macro_use]
mod plaintext;

#[macro_use]
mod rawtext;

#[macro_use]
mod rcdata;

#[macro_use]
mod script_data;

#[macro_use]
mod tag;

#[macro_use]
mod attributes;

#[macro_use]
mod comment;

#[macro_use]
mod doctype;

macro_rules! define_state_machine {
    () => {
        cdata_section_states_group!();
        data_states_group!();
        plaintext_states_group!();
        rawtext_states_group!();
        rcdata_states_group!();
        script_data_states_group!();
        tag_states_group!();
        attributes_states_group!();
        comment_states_group!();
        doctype_states_group!();
    };
}
