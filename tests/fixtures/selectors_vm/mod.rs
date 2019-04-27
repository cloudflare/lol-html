macro_rules! set {
    ($($items:expr),*) => {
        vec![$($items),*].iter().cloned().collect::<std::collections::HashSet<_>>()
    };
}

test_modules!(ast, compiler, stack);
