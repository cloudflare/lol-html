#!/bin/sh
cargo clippy --features=testing_api --all-targets && \
cargo test --features=testing_api "$@"
