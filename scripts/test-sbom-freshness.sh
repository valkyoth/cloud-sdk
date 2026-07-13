#!/usr/bin/env sh
set -eu

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cat >"$tmp/committed.json" <<'EOF'
{
  "creationInfo": {"created": "2026-01-01T00:00:00Z"},
  "documentNamespace": "https://spdx.example/first",
  "files": [{"SPDXID": "SPDXRef-File-b"}, {"SPDXID": "SPDXRef-File-a"}],
  "packages": [
    {"SPDXID": "SPDXRef-Package-b", "versionInfo": "2.0.0"},
    {"SPDXID": "SPDXRef-Package-a", "versionInfo": "1.0.0"}
  ],
  "relationships": [
    {"spdxElementId": "b", "relationshipType": "DEPENDS_ON", "relatedSpdxElement": "a"}
  ]
}
EOF

cat >"$tmp/reordered.json" <<'EOF'
{
  "creationInfo": {"created": "2026-07-13T08:00:00Z"},
  "documentNamespace": "https://spdx.example/second",
  "files": [{"SPDXID": "SPDXRef-File-a"}, {"SPDXID": "SPDXRef-File-b"}],
  "packages": [
    {"SPDXID": "SPDXRef-Package-a", "versionInfo": "1.0.0"},
    {"SPDXID": "SPDXRef-Package-b", "versionInfo": "2.0.0"}
  ],
  "relationships": [
    {"spdxElementId": "b", "relationshipType": "DEPENDS_ON", "relatedSpdxElement": "a"}
  ]
}
EOF

jq -S -f scripts/canonicalize-sbom.jq "$tmp/committed.json" \
    >"$tmp/committed.canonical.json"
jq -S -f scripts/canonicalize-sbom.jq "$tmp/reordered.json" \
    >"$tmp/reordered.canonical.json"
cmp "$tmp/committed.canonical.json" "$tmp/reordered.canonical.json"

jq '.packages[0].versionInfo = "9.9.9"' "$tmp/reordered.json" \
    >"$tmp/stale.json"
jq -S -f scripts/canonicalize-sbom.jq "$tmp/stale.json" \
    >"$tmp/stale.canonical.json"
if cmp -s "$tmp/committed.canonical.json" "$tmp/stale.canonical.json"; then
    echo "SBOM freshness test: dependency changes were ignored" >&2
    exit 1
fi

echo "2 SBOM freshness tests passed."
