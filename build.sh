#!/usr/bin/env bash

set -e

cargo build --release
mkdir -p "bin"
cp "target/release/webman" "/usr/local/bin/webman"