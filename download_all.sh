#!/usr/bin/env bash
echo "Downloading all files"
cargo run $@ --manifest-path raw_downloader/Cargo.toml
