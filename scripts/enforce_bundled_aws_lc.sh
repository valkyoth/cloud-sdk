#!/usr/bin/env sh

# This file is sourced by build and release scripts. Target-qualified AWS-LC
# controls take precedence over generic controls, so callers may not supply
# them even when they appear to request the bundled build.
for aws_lc_name in $(env | sed 's/=.*//'); do
    case "$aws_lc_name" in
    AWS_LC_SYS_USE_SYSTEM_* | AWS_LC_FIPS_SYS_USE_SYSTEM_*)
        echo "AWS-LC build policy: forbidden target-specific override: $aws_lc_name" >&2
        exit 1
        ;;
    esac
done
unset aws_lc_name

export AWS_LC_SYS_USE_SYSTEM=0
export AWS_LC_FIPS_SYS_USE_SYSTEM=0
