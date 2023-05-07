#!/usr/bin/python3
import json

total = 0
has_name = 0
has_manufacturer_name = 0
has_manufacture_country = 0
has_manufacturer_item_description = 0
has_unit_qty = 0
has_quantity = 0
has_unit_of_measure = 0
has_qty_in_package = 0
has_quantity = 0
has_unit_of_measure_price = 0
has_allow_discount = 0
has_price = 0
is_internal = 0
is_weighted = 0
for store in json.load(open("prices.json")):
    for item in store["items"]:
        total += 1
        if item['internal_code']:
            is_internal += 1
        if item['name']:
            has_name +=1
        if item['manufacturer_name']:
            has_manufacturer_name +=1
        if item['manufacture_country']:
            has_manufacture_country +=1
        if item['manufacturer_item_description']:
            has_manufacturer_item_description +=1
        if item['unit_qty']:
            has_unit_qty +=1
        if item['quantity']:
            has_quantity +=1
        if item['unit_of_measure']:
            has_unit_of_measure +=1
        if item['weighted']:
            is_weighted +=1
        if item['qty_in_package']:
            has_qty_in_package +=1
        if item['price']:
            has_price +=1
        if item['unit_of_measure_price']:
            has_unit_of_measure_price +=1
        if item['allow_discount']:
            has_allow_discount +=1


print("has_name : " + str(has_name ) + "(" + str(float(100*has_name )/float(total)) + "%)")
print("has_manufacturer_name : " + str(has_manufacturer_name ) + "(" + str(float(100*has_manufacturer_name )/float(total)) + "%)")
print("has_manufacture_country : " + str(has_manufacture_country ) + "(" + str(float(100*has_manufacture_country )/float(total)) + "%)")
print("has_manufacturer_item_description : " + str(has_manufacturer_item_description ) + "(" + str(float(100*has_manufacturer_item_description )/float(total)) + "%)")
print("has_unit_qty : " + str(has_unit_qty ) + "(" + str(float(100*has_unit_qty )/float(total)) + "%)")
print("has_quantity : " + str(has_quantity ) + "(" + str(float(100*has_quantity )/float(total)) + "%)")
print("has_unit_of_measure : " + str(has_unit_of_measure ) + "(" + str(float(100*has_unit_of_measure )/float(total)) + "%)")
print("has_qty_in_package : " + str(has_qty_in_package ) + "(" + str(float(100*has_qty_in_package )/float(total)) + "%)")
print("has_quantity : " + str(has_quantity ) + "(" + str(float(100*has_quantity )/float(total)) + "%)")
print("has_unit_of_measure_price : " + str(has_unit_of_measure_price ) + "(" + str(float(100*has_unit_of_measure_price )/float(total)) + "%)")
print("has_allow_discount : " + str(has_allow_discount ) + "(" + str(float(100*has_allow_discount )/float(total)) + "%)")
print("is_internal : " + str(is_internal ) + "(" + str(float(100*is_internal )/float(total)) + "%)")
print("is_weighted : " + str(is_weighted ) + "(" + str(float(100*is_weighted )/float(total)) + "%)")