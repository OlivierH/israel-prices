git pull
rm -rf data_raw
./download_all.sh --release
sh curate_data_raw.sh
rm -rf data
sh normalize.sh
./xml_to_csv.sh --release
./generate_processed_data.sh --release
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
echo "Pushing to git"
git push --quiet
git push origin --tags
