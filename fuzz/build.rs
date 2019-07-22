extern crate bindgen;

fn main() {
    println!("cargo:rustc-link-search=../c-api/target/debug");
    println!("cargo:rustc-link-lib=coolthing");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("../c-api/include/cool_thing.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the fuzz/bindings/bindings.rs file.
    bindings
        .write_to_file("./bindings.rs")
        .expect("Couldn't write bindings!");
}
