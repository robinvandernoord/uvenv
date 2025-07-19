#!/bin/bash
set -e

rm -f *.snap

# special build
cargo build --release --features snap

cp ./venv/bin/uv target/release/uv

snapcraft

# snapcraft upload --release=stable *.snap
