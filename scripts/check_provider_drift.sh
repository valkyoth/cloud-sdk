#!/usr/bin/env sh
set -eu

scripts/test-provider-drift-model.py
scripts/test-provider-drift-fetch.py
scripts/test-provider-drift-report.py
scripts/test-provider-drift-check.py
scripts/test-cratesio-drift.py
scripts/test-hetzner-provider-drift-bridge.py
scripts/check_hetzner_provider_drift_bridge.py
scripts/test-ovhcloud-probe.py
scripts/check_ovhcloud_probe.py

result="$(
    scripts/check_provider_drift.py \
        --plugin provider-drift/plugins/normalized-json-v1.json \
        --lock provider-drift/providers/hetzner.lock.json \
        --observation provider-drift/providers/hetzner.observed.json
)"
expected='{"changes":[],"format":"cloud-sdk-provider-drift-report/v1","provider":"hetzner","result":"clean"}'
if [ "$result" != "$expected" ]; then
    echo "provider drift: tracked clean report is not canonical" >&2
    exit 1
fi

result="$(scripts/check_cratesio_drift.py)"
expected='{"changes":[],"format":"cloud-sdk-provider-drift-report/v1","provider":"cratesio","result":"clean"}'
if [ "$result" != "$expected" ]; then
    echo "provider drift: crates.io report is not canonical" >&2
    exit 1
fi

result="$(
    scripts/check_provider_drift.py \
        --plugin provider-drift/plugins/normalized-json-v1.json \
        --lock provider-drift/providers/ovhcloud-v2-probe.lock.json \
        --observation provider-drift/providers/ovhcloud-v2-probe.observed.json
)"
expected='{"changes":[],"format":"cloud-sdk-provider-drift-report/v1","provider":"ovhcloud-v2-probe","result":"clean"}'
if [ "$result" != "$expected" ]; then
    echo "provider drift: OVHcloud probe report is not canonical" >&2
    exit 1
fi

printf '%s\n' 'Provider-generic drift evidence is canonical and clean.'
