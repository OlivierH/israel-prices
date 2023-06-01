# curl 'https://www.rami-levy.co.il/api/catalog?' \
#   --data-raw '{"store":331, }'  



# curl 'https://www.rami-levy.co.il/api/catalog?' \
#   -H 'content-type: application/json;charset=UTF-8' \
#   --data-raw '{"store":331, "d": 1240, "size": 10000}'
  
curl 'https://www.rami-levy.co.il/api/catalog?' \
  -H 'content-type: application/json;charset=UTF-8' \
  --data-raw '{"store":331, "size": 10000}'

departments: 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 1236, 1237, 1238, 1239, 1240, 1243, 1244, 1245, 1246