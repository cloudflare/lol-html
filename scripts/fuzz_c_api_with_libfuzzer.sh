#!/bin/sh

set -e

cd ./c-api && cargo +nightly build && \
cd ../fuzz && cargo +nightly build && \
cd ..  && \
rustup run nightly cargo-fuzz run --jobs 24 --release fuzz_c_api fuzz/corpus/selector_matching