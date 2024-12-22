macro_rules! ignore {
    (@info $($args:expr),+) => {
        if std::env::var("IGNORES_VERBOSE").is_ok() {
            println!($($args),+);
        }
    };

    (@total $type:expr, $count:expr) => {
        println!("Ignoring {} {} tests, run with `IGNORES_VERBOSE=1` to get more info.", $count, $type);
    };
}

mod input;

#[macro_use]
pub mod suites;

pub use self::input::Input;

pub trait TestFixture<T: std::fmt::Debug> {
    fn test_cases() -> Vec<T>;
    fn run(test: &T);
    fn run_tests() {
        for test in Self::test_cases() {
            let d = DumpOnPanic(&test);
            Self::run(&test);
            std::mem::forget(d);
        }
    }
}

struct DumpOnPanic<'a, T: std::fmt::Debug>(&'a T);
impl<T: std::fmt::Debug> Drop for DumpOnPanic<'_, T> {
    fn drop(&mut self) {
        eprintln!("test case failed: {:?}", self.0);
    }
}

