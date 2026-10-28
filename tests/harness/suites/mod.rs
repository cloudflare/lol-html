use std::fs::File;
use std::io::BufReader;

fn data_dir_path(path: &str) -> String {
    format!("{}/tests/data/{}", env!("CARGO_MANIFEST_DIR"), path)
}

fn for_each_test_file(path: &str, handler: &mut dyn FnMut(BufReader<File>)) {
    let pattern = data_dir_path(path);
    let paths: Vec<_> = glob::glob(&pattern).unwrap().collect();

    // An empty match would silently skip the whole suite (e.g. when a submodule update deletes its
    // files), so we fail instead.
    assert!(!paths.is_empty(), "no test files match `{pattern}`");

    for path in paths {
        handler(BufReader::new(File::open(path.unwrap()).unwrap()));
    }
}

fn get_test_file_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(data_dir_path(path)).unwrap())
}

pub mod html5lib_tests;
pub mod selectors_tests;
