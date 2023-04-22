#!/bin/sh

set -eu

# In case you're missing libraries, do:
# sudo apt-get install binutils-dev libunwind-dev
cargo +nightly install honggfuzz
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
    cd ./fuzz/hongg
    cargo +nightly build
    HFUZZ_INPUT=../corpus/selector_matching cargo +nightly hfuzz run hongg
)
