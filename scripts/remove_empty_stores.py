import os
import json
import csv
prices = set()
stores = []
prices_per_product = []

for root, dirs, files in os.walk("data/prices"):
    for name in files:
        prices.add(name.split(".")[0])
for root, dirs, files in os.walk("data/stores"):
    for name in files:
        chain_id = name.split("_")[0]
        with open(root + "/"+  name) as csvfile:
            reader = csv.DictReader(csvfile)
            for row in reader:
                store_id = row['store_id']
                if not (chain_id+"_"+store_id) in prices:
                    print(name, row)

# print(prices)
# full = {"prices": prices, "stores": stores,
#         "prices_per_product": prices_per_product}

# with open("data/catalog.json", "w") as outfile:
#     json.dump(full, outfile)
