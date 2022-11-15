rm -rf data_raw
sh download_all.sh
rm -rf data
sh normalize.sh
sh xml_to_csv.sh
if [ ! -d "israel-prices-data" ];
then
    git clone https://github.com/OlivierH/israel-prices-data.git
fi
cp data/* israel-prices-data -r
cd israel-prices-data
git add *
git commit -a -m"Data for `date +%F`"
git tag "`date +%F`"
git push
git push origin --tags