#!/bin/sh

set -e

(cd js-api && cargo fmt --all)
(cd c-api && cargo fmt --all)
(cd c-api/tests && cargo fmt --all)
(cd fuzz && cargo fmt --all)
cargo fmt --all && git add -u
