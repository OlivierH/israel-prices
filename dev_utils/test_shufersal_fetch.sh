RUST_LOG=israel_prices=debug cargo run --release -- \
 --no-download --no-curate \
 --load-from-json --load-item-infos-to-json --save-to-sqlite --delete-sqlite \
 --fetch-shufersal-metadata --metadata-fetch-limit 30 --save-to-sqlite-only shufersal