#!/bin/sh

set -e

cd ./fuzz/afl && \
cargo +nightly afl build && \
RUSTFLAGS="-Z sanitizer=address" cargo +nightly afl fuzz -x ../dictionaries -i ../corpus/selector_matching -o out target/debug/afl-fuzz