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

if grep -RInE '(^|[^A-Za-z0-9_])std([[:space:]]*::|[[:space:]]+as|[[:space:]]*\{|[[:space:]]*;)' crates --include='*.rs' |
    grep -Ev '^[^:]+:[0-9]+:extern crate std;' |
    grep -Ev '^[^:]+:[0-9]+:[[:space:]]*(//|///|//!|/\*)' |
    grep -Ev '^crates/cloud-sdk-reqwest/src/(asynchronous|blocking|shared)/' |
    grep -Ev '^crates/cloud-sdk-reqwest/src/test_server.rs:'; then
    echo "modularity policy: unguarded std usage found under crates/" >&2
    status=1
fi

if ! awk '
    /^#\[cfg\(feature = "async-rustls"\)\]$/ { guarded = 1; next }
    /^pub mod asynchronous;$/ {
        if (guarded) found = 1
    }
    { guarded = 0 }
    END { exit !found }
' crates/cloud-sdk-reqwest/src/lib.rs; then
    echo "modularity policy: reqwest async module lost feature guard" >&2
    status=1
fi

if ! awk '
    /^#\[cfg\(feature = "blocking-rustls"\)\]$/ { guarded = 1; next }
    /^pub mod blocking;$/ {
        if (guarded) found = 1
    }
    { guarded = 0 }
    END { exit !found }
' crates/cloud-sdk-reqwest/src/lib.rs; then
    echo "modularity policy: reqwest blocking module lost feature guard" >&2
    status=1
fi

exit "$status"
