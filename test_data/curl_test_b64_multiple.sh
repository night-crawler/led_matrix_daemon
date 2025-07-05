#!/bin/bash
set -e

clean_up () {
    ARG=$?
    rm -f image.b64
    exit $ARG
}
trap clean_up EXIT

cd "$(dirname "$0")"

render_images=()

for i in {0..15}; do
  image_file="img${i}.png"
  base64 -w0 "./$image_file" > image.b64
  IMAGE=$(cat image.b64)
  render_images+=("{\"left_image\":\"$IMAGE\",\"right_image\":\"$IMAGE\"}")
done

PAYLOAD=$(jq -n --argjson render "$(printf '[%s]' "$(IFS=,; echo "${render_images[*]}")")" \
  '{render: $render}')

echo "$PAYLOAD"

curl --unix-socket /run/led-matrix/led-matrix.sock -X POST -H "Content-Type: application/json" -d "$PAYLOAD" http://localhost/render/base64/multiple
