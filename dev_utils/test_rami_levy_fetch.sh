RUST_LOG=israel_prices=debug cargo run --release -- \
 --no-download --no-curate --no-build-item-infos \
 --load-from-json --load-item-infos-to-json \
 --fetch-rami-levy-metadata --delete-sqlite --metadata-fetch-limit 100