use cc;

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

fn main() {
    let mut build = cc::Build::new();
    for cflag in CFLAGS {
        build.flag(cflag);
    }
    build
        .debug(true)
        .opt_level(0)
        .flag_if_supported("-Wl,no-as-needed")
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .include("include")
        .include("src/deps/picotest")
        .files(&["src/deps/picotest/picotest.c", "src/test.c"])
        .compile("cool_thing_ctests");
    println!("cargo:rustc-link-lib=dylib=cool_thing_c");
}
