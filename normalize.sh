echo "Deleting empty files"
find data_raw -type f -empty -print -delete

echo "Unzipping"
gunzip data_raw/*/*.gz

echo "Add xml extension to all files that didn't have it"
ls data_raw/*/* | grep -v 'xml$' | xargs -I %  mv % %.xml

echo "Rename 'Stores...' files to 'StoresFull...'"
ls data_raw/*/* | grep -v '/StoresFull' | grep 'Stores' | xargs -I % echo mv % % | sed 's/Stores/StoresFull/2' | bash

# The numbers are chain, store, date, and ???
echo "Remove last set of digits from stores with 4 set of digits."
rename 's/([a-zA-Z]+\d+-\d+-\d+)-\d+.xml/$1.xml/' data_raw/*/*

echo "Remove date from the filename."
rename 's/([a-zA-Z]+\d+-\d+)-\d+.xml/$1.xml/' data_raw/*/*

echo "Remove date from the filename when there is no store id."
rename 's/([a-zA-Z]+\d+)--\d+.xml/$1.xml/' data_raw/*/*

echo "Convert all files to utf-8"
for f in data_raw/*/* 
do
    charset=`file -i $f | cut -d"=" -f2`
    if [ "$charset" != "utf-8" ]; then
        if [ "$charset" == "unknown-8bit" ]; then
            charset=`head -n 1 $f | cut -d'"' -f4`
        fi
        echo "Converting $f from $charset to utf-8"
        iconv -f "$charset" -t utf8 -o "$f.new" "$f"  
        mv -f "$f.new" "$f"
    fi
done

