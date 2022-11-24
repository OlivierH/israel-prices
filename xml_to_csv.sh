#!/usr/bin/env bash
echo "Converting all XML files to standard csv"
cargo run $@ --manifest-path xml_to_standard/Cargo.toml -- -i data_raw -f csv
