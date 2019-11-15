macro_rules! async_closure {
    (|$($args:tt),+| $($body:tt)*) => {
        |$($args),+| {
            Box::pin(async {
                $($body)*
            })
        }
    };
}
