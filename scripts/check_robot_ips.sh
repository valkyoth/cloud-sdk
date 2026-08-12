#!/usr/bin/env sh
set -eu

scripts/check_robot_ips.py

package_files="$(cargo package --locked -p cloud-sdk-hetzner --allow-dirty --list)"
if printf '%s\n' "$package_files" | grep -Fxq 'src/robot/server/duplicates.rs'; then
    echo "Robot IP package: obsolete server duplicate helper entered the package" >&2
    exit 1
fi
if ! printf '%s\n' "$package_files" | grep -Fxq 'src/robot/duplicates.rs'; then
    echo "Robot IP package: shared duplicate helper is missing from the package" >&2
    exit 1
fi

cargo test --locked -p cloud-sdk-hetzner --all-features robot::ip
cargo check --locked --manifest-path fuzz/Cargo.toml --bin robot_ip_response

echo "Robot IP package contains only the shared duplicate helper."
