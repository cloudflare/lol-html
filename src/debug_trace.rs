macro_rules! trace {
    ( @actions $($actions:tt)+ ) => {
        #[cfg(feature = "debug_trace")]
        println!("@action: {}", stringify!($($actions)+));
    };

    ( @chars $action_descr:expr $(, $ch:expr)* ) => {
        // NOTE: this macro expands to provided expression
        // if tracing feature is disabled. Otherwise, we would
        // need to declare intermidiate variable for the
        // character to pass it to the macro and on builds
        // with disabled tracing compiler will complain about
        // unecessary let binding.
        {
            #[cfg(feature = "debug_trace")]
            {
                print!(">{}", $action_descr);

                $({
                    use std::char;

                    print!(": {:?}", $ch.map(|ch| unsafe { char::from_u32_unchecked(ch as u32) }));
                })*

                println!();
            }

            $($ch)*
        }
    };
}
