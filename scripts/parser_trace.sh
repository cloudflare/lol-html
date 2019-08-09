#!/bin/sh
cargo run --features debug_trace,integration_test --example=parser_trace "$@"