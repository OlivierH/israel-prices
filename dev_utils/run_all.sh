RUST_LOG=israel_prices=info cargo run --release -- \
 --save-to-json --save-item-infos-to-json --save-to-sqlite --delete-sqlite --clear-files \
 --fetch-rami-levy-metadata --fetch-shufersal-metadata  --fetch-victory-metadata \
 --fetch-mega-metadata --fetch-yenot-bitan-metadata --fetch-maayan-2000-metadata \
 --fetch-am-pm-metadata --fetch-tiv-taam-metadata