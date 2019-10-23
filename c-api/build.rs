use cc::Build;

const CFLAGS: &'static [&str] = &[
    "-std=c99",
    "-pthread",
    "-Wcast-qual",
    "-Wwrite-strings",
    "-Wshadow",
    "-Winline",
    "-Wdisabled-optimization",
    "-Wuninitialized",
    "-Wcast-align",
    "-Wcast-align",
    "-Wno-missing-field-initializers",
    "-Wno-address",
];

fn build_with_flags() -> Build {
    let mut test_build = Build::new();
    for cflag in CFLAGS {
        test_build.flag(cflag);
    }
    test_build
        .debug(true)
        .opt_level(0)
        .flag_if_supported("-Wl,no-as-needed")
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true);
    test_build
}

fn main() {
    build_with_flags()
        .include("include")
        .include("tests/deps/picotest")
        .files(&["tests/deps/picotest/picotest.c", "tests/test.c"])
        .compile("cool_thing_ctests");
    println!("cargo:rustc-link-lib=dylib=cool_thing_c");
}
