#!/bin/sh

set -e

echo "===  Running library tests... ==="
cargo clippy --features=integration_test --all-targets -- -Dwarnings
cargo test --features=integration_test "$@"

echo "=== Running C API tests... ==="
prove -e 'cargo' run ::  --manifest-path=./c-api/c-tests/Cargo.toml

echo "=== Building fuzzing test case code to ensure that it uses current API... ==="
(cd fuzz/test_case && cargo check)

echo "=== Building the tooling test case code to ensure it uses the current API... ==="
(cd tools/parser_trace/ && cargo check)
(cd tools/selectors_ast/ && cargo check)
