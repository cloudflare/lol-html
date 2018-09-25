#!/bin/sh
cargo clippy --features=testing_api --all-targets -- -D warnings && \
cargo test --features=testing_api "$@"
