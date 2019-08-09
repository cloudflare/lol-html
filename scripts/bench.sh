#!/bin/sh

set -e

cargo bench --features=integration_test "$@"
open target/criterion/report/index.html