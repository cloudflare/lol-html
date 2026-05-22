#![cfg(feature = "_integration_test")]

#[macro_use]
mod harness;

mod fixtures {
    mod element_content_replacement;
    mod selector_matching;
    mod token_capturing;
}
