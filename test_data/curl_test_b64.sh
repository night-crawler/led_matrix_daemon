#!/bin/bash
set -e

clean_up () {
    ARG=$?
    rm left_image.b64 right_image.b64
    exit $ARG
}
trap clean_up EXIT

cd "$(dirname "$0")"

base64 -w0 ./img.png > left_image.b64
base64 -w0 ./img.jpg > right_image.b64

LEFT_IMAGE=$(cat left_image.b64)
RIGHT_IMAGE=$(cat right_image.b64)

PAYLOAD=$(jq -n --arg left "$LEFT_IMAGE" --arg right "$RIGHT_IMAGE" \
  '{left_image: $left, right_image: $right}')

curl --unix-socket /tmp/led-matrix.sock -X POST -H "Content-Type: application/json" -d "$PAYLOAD" http://localhost/render/base64
