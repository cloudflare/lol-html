#!/bin/sh

set -e

cargo bench --features=test_api "$@"
open target/criterion/report/index.html