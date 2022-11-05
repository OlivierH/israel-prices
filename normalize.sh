echo "Deleting empty files"
find data_raw -type f -empty -print -delete

echo "Deleting x1 files"
find data_raw -type f -name "*.x1" -print -delete

echo "Unzipping"
gunzip data_raw/*/*.gz

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

