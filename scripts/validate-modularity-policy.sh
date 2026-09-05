#!/usr/bin/env sh
set -eu

mode="${1:-check}"
if [ "$mode" != "check" ]; then
    echo "usage: scripts/validate-modularity-policy.sh check" >&2
    exit 2
fi

status=0
for root in crates/*/src/lib.rs; do
    if ! grep -Fq '#![no_std]' "$root"; then
        echo "modularity policy: missing #![no_std]: $root" >&2
        status=1
    fi
done

for root in crates/*/src/lib.rs; do
    if [ "$root" = crates/cloud-sdk-reqwest/src/lib.rs ]; then
        continue
    fi
    if ! awk '
        /^#\[cfg\(feature = "std"\)\]$/ { guarded = 1; next }
        /extern crate std;/ {
            if (!guarded) {
                print FILENAME ":" FNR ": unguarded extern crate std" > "/dev/stderr"
                bad = 1
            }
        }
        { guarded = 0 }
        END { exit bad }
    ' "$root"; then
        status=1
    fi
done

if ! awk '
    /^#\[cfg\(all\(/ { scanning = 1; std_feature = 0; supported_os = 0 }
    scanning && /feature = "std"/ { std_feature = 1 }
    scanning && /target_os = "linux"/ { supported_os = 1 }
    scanning && /^\)\)\]$/ {
        guarded = std_feature && supported_os
        scanning = 0
        next
    }
    /extern crate std;/ {
        if (guarded) found = 1
        else bad = 1
    }
    !scanning && !/extern crate std;/ { guarded = 0 }
    END { exit bad || !found }
' crates/cloud-sdk-reqwest/src/lib.rs; then
    echo "modularity policy: reqwest std import lost feature/target guard" >&2
    status=1
fi

for source in \
    crates/cloud-sdk/src/authentication/signing/tests.rs \
    crates/cloud-sdk/src/authentication/signing/tests/output.rs \
    crates/cloud-sdk-cratesio/src/credentials/tests.rs
do
    if ! awk '
        /^#\[cfg\(feature = "std"\)\]$/ { guarded = 1; next }
        /^use crate::std as test_std;$/ {
            if (guarded) found = 1
            else bad = 1
        }
        { guarded = 0 }
        END { exit bad || !found }
    ' "$source"; then
        echo "modularity policy: test std alias lost feature guard: $source" >&2
        status=1
    fi
done

if ! awk '
    /^#\[cfg\(test\)\]$/ { guarded = 1; next }
    /^mod allocation_failure;$/ {
        if (guarded) found = 1
        else bad = 1
    }
    { guarded = 0 }
    END { exit bad || !found }
' crates/cloud-sdk-hetzner/src/serde/strict_json.rs; then
    echo "modularity policy: allocation failure helper lost test guard" >&2
    status=1
fi

if grep -RInE '(^|[^A-Za-z0-9_])std([[:space:]]*::|[[:space:]]+as|[[:space:]]*\{|[[:space:]]*;)' crates --include='*.rs' |
    grep -Ev '^[^:]+:[0-9]+:extern crate std;' |
    grep -Ev '^[^:]+:[0-9]+:use crate::std as test_std;$' |
    grep -Ev '^[^:]+:[0-9]+:[[:space:]]*(//|///|//!|/\*)' |
    grep -Ev '^crates/cloud-sdk-reqwest/src/(asynchronous|blocking|shared)/' |
    grep -Ev '^crates/cloud-sdk-reqwest/src/test_server.rs:' |
    grep -Ev '^crates/cloud-sdk/tests/credential_attempt_concurrency.rs:' |
    grep -Ev '^crates/cloud-sdk/tests/response_cleanup.rs:' |
    grep -Ev '^crates/cloud-sdk/tests/encoder_cleanup.rs:' |
    grep -Ev '^crates/cloud-sdk-hetzner/src/serde/strict_json/allocation_failure.rs:' |
    grep -Ev '^crates/cloud-sdk-hetzner/tests/live_smoke(\.rs:|/)'; then
    echo "modularity policy: unguarded std usage found under crates/" >&2
    status=1
fi

if ! awk '
    /^#\[cfg\(all\(/ { scanning = 1; feature = 0; supported_os = 0 }
    scanning && /feature = "async-rustls"/ { feature = 1 }
    scanning && /target_os = "linux"/ { supported_os = 1 }
    scanning && /^\)\)\]$/ {
        guarded = feature && supported_os
        scanning = 0
        next
    }
    /^pub mod asynchronous;$/ {
        if (guarded) found = 1
        else bad = 1
    }
    !scanning && !/^pub mod asynchronous;$/ { guarded = 0 }
    END { exit bad || !found }
' crates/cloud-sdk-reqwest/src/lib.rs; then
    echo "modularity policy: reqwest async module lost feature guard" >&2
    status=1
fi

if ! awk '
    /^#\[cfg\(all\(/ {
        scanning = 1
        blocking = 0
        roots = 0
        supported_os = 0
    }
    scanning && /feature = "blocking-rustls"/ { blocking = 1 }
    scanning && /feature = "blocking-rustls-webpki-roots"/ { roots = 1 }
    scanning && /target_os = "linux"/ { supported_os = 1 }
    scanning && /^\)\)\]$/ {
        guarded = blocking && roots && supported_os
        scanning = 0
        next
    }
    /^pub mod blocking;$/ {
        if (guarded) found = 1
        else bad = 1
    }
    !scanning && !/^pub mod blocking;$/ { guarded = 0 }
    END { exit bad || !found }
' crates/cloud-sdk-reqwest/src/lib.rs; then
    echo "modularity policy: reqwest blocking module lost feature guard" >&2
    status=1
fi

exit "$status"
