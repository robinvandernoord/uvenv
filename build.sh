#!/bin/bash
set -e

rm -rf target/wheels/

maturin build --release --target aarch64-unknown-linux-gnu
maturin build --release --target x86_64-unknown-linux-gnu
maturin sdist

maturin upload --skip-existing -u __token__ target/wheels/*
