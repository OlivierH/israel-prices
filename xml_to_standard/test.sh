rm -rf output_test
cargo run -- -i data_test -o output_test -p -f csv
diff -r output_test/ golden_data_csv/