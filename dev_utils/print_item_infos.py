#!/usr/bin/python3
import json
for i in json.load(open("item_infos.json"))["data"]:
    print(i)