#!/usr/bin/env bash
set -euo pipefail

t32="Tabel32 Nationaliteitentabel (gesorteerd op code).csv"
t33="Tabel33 Gemeententabel (gesorteerd op code).csv"
t34="Tabel34 Landentabel (gesorteerd op code).csv"

curl "https://publicaties.rvig.nl/dsresource?objectid=73106c8d-a666-4e30-a8e0-80b748a15d24" > "$t32"
curl "https://publicaties.rvig.nl/dsresource?objectid=86360a47-728a-4c10-9b95-a4ea9fcaf680" > "$t33"
curl "https://publicaties.rvig.nl/dsresource?objectid=29b66cb2-02ef-4a11-baf4-316ae00d8fa1" > "$t34"

for table in "$t32" "$t33" "$t34"; do 
	# Some version of iconv can't update the file in place
	# TODO: determine if we lose anything with the conversion to utf-8
	iconv -f utf-16le -t utf-8 "$table" > "temp.csv"
	dos2unix "temp.csv"
	mv "temp.csv" "$table"
done

# BSD sed can't do this in place, so create and remove backup file
sed -i.deleteme 's/,94.10 Landcode/94.10 Landcode/' "$t34"
rm *deleteme

# TODO
echo "ab35e37f74b4f9ad69571b6726bfe8c380b694b18b24ee87343423deae990bd7  $t32" | sha256sum -c
echo "a584ae747adb7058f81d356744cd1f6996943320451aaf251d02e4da109f280c  $t33" | sha256sum -c
echo "26ed3a9d93b4e6729b096cbe790ed2b181cfb6ba4ffb6897b2fb1d0a4b0f2a12  $t34" | sha256sum -c
