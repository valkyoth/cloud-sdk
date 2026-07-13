#!/usr/bin/env sh
set -eu

mkdir -p sbom
cargo sbom --output-format spdx_json_2_3 > sbom/cloud-sdk.spdx.json
cargo sbom --project-directory tests/reqwest-feature-unification \
    --output-format spdx_json_2_3 > sbom/reqwest-feature-unification.spdx.json
test -s sbom/cloud-sdk.spdx.json
test -s sbom/reqwest-feature-unification.spdx.json
