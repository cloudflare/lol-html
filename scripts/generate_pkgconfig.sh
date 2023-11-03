#!/bin/sh

set -eu

cat << EOF > lol-html.pc
prefix=${PREFIX:-/usr}
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: lol-html
Description: Low output latency streaming HTML parser/rewriter
Version: $(cargo metadata --format-version=1 --no-deps --manifest-path=c-api/Cargo.toml | jq -r '.packages[0].version')

Requires:
Libs: -L\${libdir} -llolhtml
Cflags: -I\${includedir}
EOF
