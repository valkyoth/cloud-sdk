#!/usr/bin/env sh
set -eu

for script in scripts/*.sh; do
    sh -n "$script"
done
