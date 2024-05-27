#!/bin/bash
set -e
curl --unix-socket /tmp/led-matrix.sock -X POST \
  -F "file1=@./img.png" \
  -F "file2=@./img.jpg" \
  -F "file3=@./img.jpg" \
  -F "file4=@./img.jpg" \
  -F "file5=@./img.jpg" \
  -F "file6=@./img.jpg" \
  -F "file7=@./img.jpg" \
  -F "file8=@./img.jpg" \
  -F "file9=@./img.jpg" \
  -F "file10=@./img.jpg" \
  http://localhost/render/files
