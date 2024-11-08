#!/bin/sh

set -e

cargo +nightly install honggfuzz

cd ./fuzz/hongg && \
cargo +nightly build && \
RUSTFLAGS="-Z sanitizer=address" HFUZZ_RUN_ARGS="--threads=4" HFUZZ_INPUT=../corpus/selector_matching cargo +nightly hfuzz run hongg
