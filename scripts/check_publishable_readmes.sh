#!/usr/bin/env sh
set -eu

root_dir="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)"

if [ "$#" -eq 0 ]; then
    set -- "$root_dir/README.md" "$root_dir"/crates/*/README.md
fi

stale_pattern='implementation stop reached|pentest required|latest published (release|provider release)|preparing the workspace .* release|planned provider'

for readme in "$@"; do
    if [ ! -f "$readme" ]; then
        echo "publishable README check: missing file: $readme" >&2
        exit 2
    fi
    if grep -Eiq "$stale_pattern" "$readme"; then
        echo "publishable README check: development-only release status in $readme" >&2
        grep -Ein "$stale_pattern" "$readme" >&2
        exit 1
    fi
done
