#!/usr/bin/env sh
set -eu

status=0
for file in $(find crates scripts -type f \( -name '*.rs' -o -name '*.sh' \)); do
    lines=$(wc -l < "$file")
    if [ "$lines" -gt 500 ]; then
        echo "file length policy: $file has $lines lines; limit is 500" >&2
        status=1
    fi
done
exit "$status"
