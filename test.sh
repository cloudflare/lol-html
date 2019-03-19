#!/bin/sh
cd c-api && cargo clippy && cargo test && cd ../  \
cargo clippy --features=test_api --all-targets && \
cargo test --features=test_api "$@"
