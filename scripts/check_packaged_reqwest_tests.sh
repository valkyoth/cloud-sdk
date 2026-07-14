#!/usr/bin/env sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"
export AWS_LC_FIPS_SYS_USE_SYSTEM=0

package_id="$(cargo pkgid -p cloud-sdk-reqwest)"
package_version="${package_id##*#}"
package_version="${package_version##*@}"
package_name="cloud-sdk-reqwest-${package_version}"
archive="target/package/${package_name}.crate"

cargo package -p cloud-sdk-reqwest --allow-dirty --all-features \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"'

temporary="$(mktemp -d "${TMPDIR:-/tmp}/cloud-sdk-package.XXXXXX")"
trap 'rm -rf -- "$temporary"' EXIT HUP INT TERM
tar -xzf "$archive" -C "$temporary"

cargo test \
    --manifest-path "$temporary/$package_name/Cargo.toml" \
    --locked \
    --no-run \
    --features blocking-rustls-fips \
    --config "patch.crates-io.cloud-sdk.path=\"$root/crates/cloud-sdk\"" \
    --config "patch.crates-io.cloud-sdk-sanitization.path=\"$root/crates/cloud-sdk-sanitization\""

printf '%s\n' "Packaged cloud-sdk-reqwest FIPS tests compiled successfully."
