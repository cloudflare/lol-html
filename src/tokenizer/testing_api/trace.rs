macro_rules! trace {
    ( @actions $($actions:tt)+ ) => {
        #[cfg(feature = "trace_actions")]
        println!("@action: {}", stringify!($($actions)+));
    };

    ( @chars $ch:ident ) => {
        #[cfg(feature = "trace_char")]
        {
            use std::char;

            println!(">ch: {:?}", $ch.map(|ch| unsafe { char::from_u32_unchecked(ch as u32) }));
        }
    };

    ( @raw $self:tt, $end_pos:expr ) => {
        #[cfg(feature = "trace_raw")]
        {
            use std::fmt::Write;

            let mut chunk = unsafe { String::from_utf8_unchecked($self.buffer.to_vec()) };
            let mut start = String::new();
            let mut end = String::new();

            write!(start, "|{}|", $self.raw_start).unwrap();
            write!(end, "|{}|", $end_pos).unwrap();

            chunk.insert_str($end_pos, &end);
            chunk.insert_str($self.raw_start, &start);

            println!("--Token raw slice--");
            println!("{}", chunk);
        }
    };
}
