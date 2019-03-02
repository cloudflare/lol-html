#!/bin/sh
cargo clippy --features=test_api --all-targets && \
cargo test --features=test_api "$@"
