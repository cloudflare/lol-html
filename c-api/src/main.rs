extern "C" {
    fn run_tests() -> usize;
}

fn main() {
    unsafe {
        run_tests();
    }
}
