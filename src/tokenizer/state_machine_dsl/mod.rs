#[macro_use]
mod action;

#[macro_use]
mod action_list;

#[macro_use]
mod state_body;

#[macro_use]
mod state;

#[macro_use]
mod arm_pattern;

#[macro_use]
mod condition;

#[macro_use]
mod input;

macro_rules! define_state_group {
    ( $name:ident = { $($states:tt)+ } ) => {
        macro_rules! $name {
            () => {
                impl<H: LexUnitHandler> Tokenizer<H>
                {
                    state!($($states)+);
                }
            };
        }
    };
}
