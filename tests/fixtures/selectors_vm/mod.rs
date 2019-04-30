macro_rules! set {
    ($($items:expr),*) => {
        vec![$($items),*].iter().cloned().collect::<std::collections::HashSet<_>>()
    };
}

macro_rules! map {
    ($($items:expr),*) => {
        vec![$($items),*].iter().cloned().collect::<std::collections::HashMap<_, _>>()
    };
}

test_modules!(ast, compiler, stack, execution);
