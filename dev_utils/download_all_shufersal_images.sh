sqlite3 data.sqlite "SELECT ItemCode, ImageUrl FROM ShufersalMetadata ORDER BY 1" | tr "|" " " | grep https | awk '{print "images/shufersal/"$1".png " $2}' | xargs -n 2 -P 4 -t wget -O

# A lot of images are just blank squares. To remove them, delete all files of less than 200 bytes.
find images/shufersal/ -type f  -size -200 -delete
