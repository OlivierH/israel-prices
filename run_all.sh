echo "Downloading all files"
cargo run --manifest-path raw_downloader/Cargo.toml

echo "Unzipping"
gunzip data_raw/*/*.gz

echo "Add xml extension to all files that didn't have it"
ls data_raw/*/* | grep -v 'xml$' | xargs -I %  mv % %.xml

echo "Rename 'Stores...' files to 'StoresFull'"
ls data_raw/*/* | grep -v '/StoresFull' | grep 'Stores' | xargs -I % echo mv % % | sed 's/Stores/StoresFull/2' | bash

echo "Convert all files to utf-8"
for f in data_raw/*/* 
do
    charset=`file -i $f | cut -d"=" -f2`
    if [ "$charset" != "utf-8" ]; then
        iconv -f "$charset" -t utf8 -o "$f.new" "$f"  
        mv -f "$f.new" "$f"
    fi
done

# ls data_raw/*/* | grep '^' | xargs -I %  mv % %.xml