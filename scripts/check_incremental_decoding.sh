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

if grep -R -q -- 'BTreeSet\|SecretString::with_capacity' "$root"; then
    echo "incremental decoding: parser staging uses infallible allocation" >&2
    exit 1
fi
for contract in \
    try_append_secret_string \
    try_reserve \
    visitor_panic_permanently_poisons_the_decoder \
    valid_seed_reaches_complete_incremental_and_independent_parsers \
    'serde_json::from_slice'; do
    if ! grep -R -q -- "$contract" \
        "$root" crates/cloud-sdk-sanitization/src fuzz/fuzz_targets/incremental_json.rs \
        fuzz/tests/incremental_json_seeds.rs; then
        echo "incremental decoding: missing remediation contract $contract" >&2
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

cargo test --locked -p cloud-sdk-hetzner --features serde,std incremental

echo "incremental decoding: contract, limits, chunking, cleanup, and docs passed"
