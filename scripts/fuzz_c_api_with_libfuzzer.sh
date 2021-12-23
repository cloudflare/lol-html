#!/bin/sh

set -e

cd ./c-api && cargo +nightly build
cd ../fuzz && cargo +nightly build
cd ..
# add `liblolhtml.so` to linker path
export LD_LIBRARY_PATH="$(realpath ${CARGO_TARGET_DIR:-c-api/target}/debug/deps)"
rustup run nightly cargo-fuzz run --jobs 24 --release fuzz_c_api fuzz/corpus/selector_matching