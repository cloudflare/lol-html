#!/bin/sh

export CARGO_HOME=/var/lib/cargo
export CARGO_TARGET_DIR=$CARGO_HOME/target

set -e

# Series of hacks to prebuild dependencies in a cached layer
# (workaround for https://github.com/rust-lang/cargo/issues/2644)

# Create dummy sources for our library
mkdir -p src c-api/src /var/lib/cargo/target/debug/build
touch src/lib.rs c-api/src/lib.rs

# Workaround to stop pre-commit complaining about missing project root
mkdir -p /var/lib/cargo/target/debug/build
touch /var/lib/cargo/target/debug/build/Cargo.toml
rm -rf examples/*

mkdir -p benches/deps/lazyhtml/rust/src
mkdir -p benches/deps/lazyhtml/rust/lazyhtml-sys/src
touch benches/deps/lazyhtml/rust/src/lib.rs benches/deps/lazyhtml/rust/lazyhtml-sys/src/lib.rs

mkdir -p tests
mkdir -p benches/deps/lazyhtml/rust/tests
mkdir -p benches/deps/lazyhtml/rust/benches

echo 'fn main() {}' | tee \
 benches/bench.rs \
 tests/main.rs \
 benches/deps/lazyhtml/rust/tests/test.rs \
 benches/deps/lazyhtml/rust/benches/bench.rs

# Build library with Cargo.lock (including all the dependencies)
# and then clean artifacts of the library itself but keep prebuilt deps
cargo test --no-run --locked --all && cargo clean --locked
cd c-api && cargo build --locked --all && cargo clean --locked