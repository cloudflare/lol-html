#!/bin/sh

set -eu

cargo +nightly install afl
(
	cd ./c-api
	# ok to use default toolchain here since C ABI is stable
	cargo build
)
LOLHTML_LIB_PATH="$(realpath ${CARGO_TARGET_DIR:-c-api/target}/debug/deps)"
# add `liblolhtml.so` to compiling linker path
export RUSTFLAGS="-L $LOLHTML_LIB_PATH"
# add `liblolhtml.so` to runtime linker path
export LD_LIBRARY_PATH="$LOLHTML_LIB_PATH"
(
    cd ./fuzz/afl
    cargo +nightly afl build
    cargo +nightly afl fuzz -x ../dictionaries -i ../corpus/selector_matching -o out target/debug/afl-fuzz
)