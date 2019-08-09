#!/bin/sh

set -e

cargo clippy --features=integration_test --all-targets
cargo test --features=integration_test "$@"
make -C c-api test
