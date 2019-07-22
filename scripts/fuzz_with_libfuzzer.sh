#!/bin/sh

set -e

cd ./fuzz/ && \
cargo +nightly build && \
cd .. & \
rustup run nightly cargo-fuzz run --jobs 24 --release fuzz_rewriter fuzz/corpus/selector_matching