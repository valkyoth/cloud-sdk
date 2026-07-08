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

if grep -RInE '(^|[^A-Za-z0-9_])std::|extern crate std' crates --include='*.rs' |
    grep -v '#[cfg(feature = "std")]' |
    grep -v 'extern crate std'; then
    echo "modularity policy: unguarded std usage found under crates/" >&2
    status=1
fi

exit "$status"
