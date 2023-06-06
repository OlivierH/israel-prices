curl 'https://shop.hazi-hinam.co.il/proxy/api/Catalog/get' \
  -H 'Content-Type: application/json' \
  --compressed


# need cookie
curl 'https://shop.hazi-hinam.co.il/proxy/api/item/getItemsBySubCategory?Id=11271' \
  -H 'Content-Type: application/json' \
  --compressed
curl 'https://shop.hazi-hinam.co.il/proxy/api/item/getItemsBySubCategory?Id=11271' \
  -H 'Content-Type: application/json; charset=utf-8' \
  -H 'Cookie: H_UUID=bae8954a-9310-40ef-abd1-af2360bcd599; H_Authentication=%7B%22access_token%22%3A%22CA455F57E2796A8FCF5ABA81634BF9414465DAB274E8AE98A58417E95C7E9656%22%2C%22expires_in%22%3A172800.0%2C%22error%22%3Anull%7D; HR=EF880BB55257ABC388E120A3C61F0F3DE071DB9766F9957F865210A0F8C6668D' \
  --compressed