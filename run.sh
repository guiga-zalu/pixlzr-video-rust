entrada="$1"
saida="$2"
BLOCK_SIZE="$3"
K="$4"

temp_video="./temp-0001.mp4"

cargo r --release -- \
    "$entrada" \
    "$temp_video" \
    "$BLOCK_SIZE" \
    "$K"

ffmpeg -y \
    -i "$entrada" \
    -i "$temp_video" \
    -c:v copy -map 0:a -map 1:v \
    "$saida"

rm "$temp_video"

#mpv "$saida"
