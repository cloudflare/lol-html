#!/bin/sh

set -eu

cargo +nightly install cargo-fuzz
(
	cd ./c-api
	# ok to use default toolchain here since C ABI is stable
	cargo build
)
# add `liblolhtml.so` to linker path
export LD_LIBRARY_PATH="$(realpath ${CARGO_TARGET_DIR:-c-api/target}/debug/deps)"
(
	cd ./fuzz
	cargo +nightly build
)
rustup run nightly cargo-fuzz run --jobs 24 --release fuzz_rewriter fuzz/corpus/selector_matching
