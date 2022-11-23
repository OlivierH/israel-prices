rm -rf data_raw
sh download_all.sh
sh curate_data_raw.sh
rm -rf data
sh normalize.sh
sh xml_to_csv.sh
sh generate_processed_data.sh
python3 scripts/build_catalog.py
if [ ! -d "israel-prices-data" ];
then
    git clone git@github.com:OlivierH/israel-prices-data.git
fi
cp data/* israel-prices-data -r
cd israel-prices-data
git add *
git commit -a -m"Data for `date +%F`"
git tag "`date +%F`"
git push
git push origin --tags
