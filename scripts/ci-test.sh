#!/bin/sh

# Required for the cargo build artifacts caching hack
export CARGO_HOME=/var/lib/cargo
export CARGO_TARGET_DIR=$CARGO_HOME/target

set -e

sudo cargo clippy --features=test_api --all-targets
# We don't use cargo-to-teamcity because it requires verbose output
# for the tests and TeamCity gets overwhelmed by the output it gets
# for 300k+ tests and it takes eternity (30+ min vs ~10 min) to dump
# and process the whole output. So, raw build logs FTW.
sudo cargo test --features=test_api
sudo make -C c-api test

