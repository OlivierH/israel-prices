echo "Converting all XML files to standard json"
cargo run --manifest-path xml_to_standard/Cargo.toml -- -i data_raw -p
