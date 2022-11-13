sh download_all.sh
sh normalize.sh
sh xml_to_csv.sh
if [ ! -d "israel-prices-data" ];
then
    git clone https://github.com/OlivierH/israel-prices-data.git
fi
cp data_csv/* israel-prices-data -r
cd israel-prices-data
git tag "`date +%F`"
git commit -a -m"Data for `date +%F`"
git push