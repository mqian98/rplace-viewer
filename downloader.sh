#!/opt/homebrew/bin/zsh

for i in {0..78}
do
    n=$(printf %02d $i)
    csvfilename="2022_place_canvas_history-0000000000$n.csv"
    gzipfilename="$csvfilename.gzip"
    gzfilename="$csvfilename.gz"
    url="https://placedata.reddit.com/data/canvas-history/$gzipfilename"
    downloadfilename="data/$gzfilename"

    curl $url -o $downloadfilename
    gunzip $downloadfilename 
done
