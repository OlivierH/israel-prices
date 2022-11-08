echo "Converting all XML files to standard json"
cargo run --manifest-path xml_to_json/Cargo.toml -- data_raw/*/*
