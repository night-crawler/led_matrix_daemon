#!/bin/bash
set -e

cd "$(dirname "$0")"

# Create an array to hold the form data
form_data=()

# Loop through indices 0 to 15 and add each file twice
for i in {0..15}; do
  form_data+=("-F" "file-a-${i}=@./img${i}.png" "-F" "file-b-${i}=@./img${i}.png")
done

# Execute the curl command with the form data
curl --unix-socket /var/run/led-matrix/led-matrix.sock -X POST "${form_data[@]}" http://localhost/render/files
