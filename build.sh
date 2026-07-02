#!/usr/bin/env bash

set -e

cargo build --release
mkdir -p "bin"
sudo cp "target/release/webman" "/usr/local/bin/webman"