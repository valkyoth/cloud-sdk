#!/usr/bin/env sh
set -eu

root="crates/cloud-sdk-hetzner/src/serde/incremental"

for symbol in \
    IncrementalJsonDecoder \
    IncrementalJsonLimits \
    IncrementalJsonVisitor \
    IncrementalJsonEvent \
    IncrementalJsonProgress; do
    if ! grep -R -q -- "$symbol" "$root"; then
        echo "incremental decoding: missing public contract $symbol" >&2
        exit 1
    fi
done

for document in \
    docs/INCREMENTAL_DECODING.md \
    docs/MIGRATION_0.49.0.md \
    docs/PUBLIC_API_REVIEW_0.49.0.md \
    docs/DEPENDENCY_REVIEW_0.49.0.md \
    release-notes/RELEASE_NOTES_0.49.0.md; do
    if [ ! -s "$document" ]; then
        echo "incremental decoding: missing evidence $document" >&2
        exit 1
    fi
done

find "$root" -type f -name '*.rs' -print | while IFS= read -r source; do
    lines="$(wc -l < "$source")"
    if [ "$lines" -gt 500 ]; then
        echo "incremental decoding: $source exceeds 500 lines" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk-hetzner --features serde incremental

echo "incremental decoding: contract, limits, chunking, cleanup, and docs passed"
