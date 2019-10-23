#!/bin/sh

set -e

echo "===  Running library tests... ==="
cargo clippy --features=integration_test --all-targets
cargo test --features=integration_test "$@"

echo "=== Running C API tests... ==="
cargo build --manifest-path=./c-api/Cargo.toml && prove -v -e 'sh' ./scripts/c_api_test.sh

echo "=== Building fuzzing test case code to ensure that it uses recent API... ==="
(cd fuzz/test_case && cargo build)
