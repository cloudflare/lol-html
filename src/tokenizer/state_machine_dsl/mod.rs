#[macro_use]
mod action;

#[macro_use]
mod action_list;

#[macro_use]
mod state_body;

#[macro_use]
mod state;

macro_rules! define_state_group {
    ( $name:ident = { $($states:tt)+ } ) => {
        macro_rules! $name {
            () => {
                impl<'t, H: FnMut(LexResult)> Tokenizer<'t, H> {
                    state!($($states)+);
                }
            };
        }
    };
}
