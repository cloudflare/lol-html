#!/bin/sh

set -e

echo "===  Running library tests... ==="
cargo clippy --features=integration_test --all-targets
cargo test --features=integration_test "$@"

echo "=== Running C API tests... ==="
make -C c-api test

echo "=== Building fuzzing test case code to ensure that it uses recent API... ==="
(cd fuzz/test_case && cargo build)
