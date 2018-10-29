#!/bin/sh
cargo bench --features=testing_api "$@" && \
open target/criterion/report/index.html