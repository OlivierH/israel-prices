import os
import csv

data = set()

for root, dirs, files in os.walk("data/prices"):
    for name in files:
        filename = os.path.join(root, name)
        # print("> ", filename)
        parts = filename.split("/")[-1].split(".")[0].split("_")
        chain = parts[0]
        store = parts[1]
        with open(filename) as csvfile:
            reader = iter(csv.reader(csvfile))
            next(reader)
            for row in reader:
                if row[1] == "false":
                    continue
                id = row[0]
                name = row[2]
                data.add((chain, store, id, name))

for s in data:
    print(s[0], s[1], s[2], s[3])
