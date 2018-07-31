macro_rules! debug {
    ( @trace_actions $($actions:tt)+ ) => {
        #[cfg(feature = "trace_actions")]
        println!("@action: {}", stringify!($($actions)+));
    };

    ( @trace_char $ch:ident ) => {
        #[cfg(feature = "trace_char")]
        {
            use std::char;
            println!(">ch: {:?}", $ch.map(|ch| unsafe { char::from_u32_unchecked(ch as u32) }));
        }
    };
}
