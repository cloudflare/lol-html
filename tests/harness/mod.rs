mod decoder;
mod parsing_result;
pub mod test;
pub mod token;
mod unescape;

macro_rules! read_tests {
    ($path:expr) => {
        glob::glob(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/", $path))
            .unwrap()
            .map(|path| BufReader::new(File::open(path.unwrap()).unwrap()))
    };
}
