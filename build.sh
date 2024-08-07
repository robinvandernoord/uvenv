#!/bin/bash
set -e

rm -rf target/wheels/

# use --zig for better cross compatibility
maturin build --release --strip --target aarch64-unknown-linux-gnu --zig
maturin build --release --strip --target x86_64-unknown-linux-gnu --zig
maturin sdist

maturin upload --skip-existing -u __token__ target/wheels/*
