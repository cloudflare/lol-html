#!/bin/sh

set -e

cargo bench --features "integration_test lhtml" "$@"
open target/criterion/report/index.html