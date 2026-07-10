# IANA IPv6 Source Lock

Status: source-locked for `v0.11.0`.

Retrieved: 2026-07-10

## Sources

| Registry | Last updated | SHA-256 of CSV |
| --- | --- | --- |
| [IPv6 Global Unicast Address Space](https://www.iana.org/assignments/ipv6-unicast-address-assignments/ipv6-unicast-address-assignments.xhtml) | 2025-10-10 | `ebff425bb1acbbea29c4f28146930873faddd5ee57260e95b57ed9e04ea21dd8` |
| [IPv6 Special-Purpose Address Space](https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry.xhtml) | 2025-10-09 | `775feea0621dec8735a44fbf30f762e721e8f0a1b3ab7eb341961a88cfce2139` |

Machine-readable sources:

- <https://www.iana.org/assignments/ipv6-unicast-address-assignments/ipv6-unicast-address-assignments.csv>
- <https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry-1.csv>

The public Load Balancer server-target policy admits only the following
allocated global-unicast prefixes. The machine-readable copy is
`docs/IANA_IPV6_GLOBAL_UNICAST.tsv`:

```text
2001:200::/23  2001:400::/23  2001:600::/23  2001:800::/22
2001:c00::/23  2001:e00::/23   2001:1200::/23 2001:1400::/22
2001:1800::/23 2001:1a00::/23 2001:1c00::/22 2001:2000::/19
2001:4000::/23 2001:4200::/23 2001:4400::/23 2001:4600::/23
2001:4800::/23 2001:4a00::/23 2001:4c00::/23 2001:5000::/20
2001:8000::/19 2001:a000::/20 2001:b000::/20 2003::/18
2400::/12      2410::/12      2600::/12      2610::/23
2620::/23      2630::/12      2800::/12      2a00::/12
2a10::/12      2c00::/12
```

The partially allocated IETF block `2001::/23`, 6to4 `2002::/16`,
documentation `2001:db8::/32`, and AS112 service `2620:4f:8000::/48` are not
ordinary server-address provenance and are rejected. Unlisted portions of
`2000::/3` remain reserved by IANA and are rejected by default.

Any registry update requires review of both tables, updates to this lock and
the Rust prefix table, regression tests, and a new pentest-reviewed commit.

Run the offline synchronization check during development:

```bash
scripts/check_iana_ipv6_registry.py --local-only
```

The `v0.11.0` release gate runs the live registry check:

```bash
scripts/check_iana_ipv6_registry.py --fetch
```

The live check is intentionally fail-closed. Any changed registry digest,
allocation row, or required special-purpose range requires manual policy
review before the pinned hashes and prefix lock are updated.
