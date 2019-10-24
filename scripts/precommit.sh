#!/bin/sh

set -e

(cd c-api/cool-thing-c && cargo fmt --all)
(cd c-api/ctests && cargo fmt --all)
(cd fuzz && cargo fmt --all)
cargo fmt --all && git add -u
