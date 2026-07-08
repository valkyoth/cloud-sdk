#!/usr/bin/env sh
set -eu

cloud_url="https://docs.hetzner.cloud/cloud.spec.json"
cloud_sha256="9ca6b542a057b002804b9f4f45ccfdb8b9a28c92b7e5bf5ae1b7f46b54fe0093"
hetzner_url="https://docs.hetzner.cloud/hetzner.spec.json"
hetzner_sha256="f70750016d81c927ddf877e103541c90d3e3372723cdf54e6fd7b2eba4a8108a"

require_text() {
    file="$1"
    text="$2"
    if ! grep -Fq "$text" "$file"; then
        echo "$file is missing required text: $text" >&2
        exit 1
    fi
}

check_local_lock() {
    test -s docs/API_MATRIX.md
    test -s docs/SPEC_LOCK.md
    test -s docs/API_FINGERPRINTS.tsv
    test -s docs/API_SCHEMA_FINGERPRINTS.tsv

    require_text docs/SPEC_LOCK.md "$cloud_url"
    require_text docs/SPEC_LOCK.md "$cloud_sha256"
    require_text docs/SPEC_LOCK.md "$hetzner_url"
    require_text docs/SPEC_LOCK.md "$hetzner_sha256"
    require_text docs/SPEC_LOCK.md "Total source-locked operations: 221"
    require_text docs/SPEC_LOCK.md "2026-07-08"

    require_text docs/API_MATRIX.md "Total source-locked operations: 221"
    require_text docs/API_MATRIX.md "| cloud | Actions | GET | \`/actions\` | \`get_actions\` |"
    require_text docs/API_MATRIX.md "| cloud | Data Centers | GET | \`/datacenters\` | \`list_datacenters\` |"
    require_text docs/API_MATRIX.md "| cloud | Zone RRSet Actions | POST | \`/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/change_ttl\` |"
    require_text docs/API_MATRIX.md "| hetzner | Storage Boxes | GET | \`/storage_boxes\` | \`list_storage_boxes\` |"
    require_text docs/API_MATRIX.md "| hetzner | Storage Box Snapshots | POST | \`/storage_boxes/{id}/snapshots\` |"

    require_text docs/API_FINGERPRINTS.tsv "cloud	GET	/actions	Actions	get_actions"
    require_text docs/API_FINGERPRINTS.tsv "hetzner	GET	/storage_boxes	Storage Boxes	list_storage_boxes"
    require_text docs/API_SCHEMA_FINGERPRINTS.tsv "cloud	ServiceHTTPProtocol"
}

check_fetch_lock() {
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

    curl -fsSL "$cloud_url" -o "$tmp_dir/cloud.spec.json"
    curl -fsSL "$hetzner_url" -o "$tmp_dir/hetzner.spec.json"

    printf '%s  %s\n' "$cloud_sha256" "$tmp_dir/cloud.spec.json" | sha256sum -c -
    printf '%s  %s\n' "$hetzner_sha256" "$tmp_dir/hetzner.spec.json" | sha256sum -c -
}

mode="${1:---local-only}"
case "$mode" in
--local-only)
    check_local_lock
    ;;
--fetch)
    check_local_lock
    check_fetch_lock
    ;;
*)
    echo "usage: scripts/check_hetzner_upstream.sh [--local-only|--fetch]" >&2
    exit 2
    ;;
esac
