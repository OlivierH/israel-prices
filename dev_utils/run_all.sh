RUST_LOG=israel_prices=info cargo run --release -- \
 --save-to-json --save-item-infos-to-json --save-to-sqlite --delete-sqlite --clear-files \
 --fetch-rami-levy-metadata --fetch-shufersal-metadata  --fetch-victory-metadata \
 --fetch-mega-metadata --fetch-yenot-bitan-metadata