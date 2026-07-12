#!/usr/bin/env sh
set -eu

supported_versions="1.90.0 1.91.0 1.92.0 1.93.0 1.94.0 1.95.0 1.96.0 1.96.1 1.97.0"
requested="${1:-}"

check_version() {
    version="$1"
    case " $supported_versions " in
    *" $version "*) ;;
    *)
        echo "Rust matrix: unsupported version $version" >&2
        exit 2
        ;;
    esac
    cargo "+$version" check --workspace --all-features
}

if [ -n "$requested" ]; then
    check_version "$requested"
    exit 0
fi

for version in $supported_versions; do
    check_version "$version"
done
