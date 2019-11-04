#!/bin/sh

set -e

cargo bench "$@"
open target/criterion/report/index.html