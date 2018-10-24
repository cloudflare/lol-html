macro_rules! trace {
    ( @actions $($actions:tt)+ ) => {
        #[cfg(feature = "trace_actions")]
        println!("@action: {}", stringify!($($actions)+));
    };

    ( @chars $action_descr:expr $(, $ch:expr)* ) => {
        {
            #[cfg(feature = "trace_char")]
            {
                print!("{}", $action_descr);

                $({
                    use std::char;

                    print!(": {:?}", $ch.map(|ch| unsafe { char::from_u32_unchecked(ch as u32) }));
                })*

                println!();
            }

            $($ch)*
        }
    };

    ( @raw $self:tt, $input:ident, $end_pos:expr ) => {
        #[cfg(feature = "trace_raw")]
        {
            use std::fmt::Write;

            let mut input = $input.as_bytes().as_string();
            let mut start = String::new();
            let mut end = String::new();

            write!(start, "|{}|", $self.lex_unit_start).unwrap();
            write!(end, "|{}|", $end_pos).unwrap();

            input.insert_str($end_pos, &end);
            input.insert_str($self.lex_unit_start, &start);

            println!("--Token raw slice--");
            println!("{}", input);
        }
    };
}
