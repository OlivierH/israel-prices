import os
import json

prices = []
stores = []
prices_per_product = []

for root, dirs, files in os.walk("data/prices"):
    for name in files:
        prices.append(name)
for root, dirs, files in os.walk("data/stores"):
    for name in files:
        stores.append(name)
for root, dirs, files in os.walk("data/prices_per_product"):
    for name in files:
        prices.append(prices_per_product)

full = {"prices": prices, "stores": stores,
        "prices_per_product": prices_per_product}

with open("data/catalog.json", "w") as outfile:
    json.dump(full, outfile)
