#!/usr/bin/env sh
set -eu

exec python3 scripts/check_doc_links.py --root .
