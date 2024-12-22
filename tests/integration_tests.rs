#![cfg(feature = "integration_test")]

#[macro_use]
mod harness;

mod fixtures {
    mod token_capturing;
    mod selector_matching;
    mod element_content_replacement;
}
