#!/bin/sh
cargo bench --features=test_api "$@" && \
open target/criterion/report/index.html