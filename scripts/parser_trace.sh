#!/bin/sh
cargo run --features debug_trace,test_api --example=parser_trace "$@"