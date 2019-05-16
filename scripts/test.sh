#!/bin/sh

set -e

cargo clippy --features=test_api --all-targets
cargo test --features=test_api "$@"
make -C c-api test
