#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

# Candidate checks do not attest pentest acceptance or authorize publication.
reviewed_head="$(git rev-parse HEAD)"
test -z "$(git status --porcelain --untracked-files=all)"
scripts/check_cratesio_qualification.sh
scripts/check_hetzner_api_surface.sh --fetch
scripts/check_latest_tools.sh --fetch
scripts/check_exact_dependency_pins.py --fetch
python3 scripts/check_cratesio_endpoints.py --fetch
python3 scripts/check_cratesio_request_policy.py --fetch
python3 scripts/check_candidate_reproduction.py
test "$(git rev-parse HEAD)" = "$reviewed_head"
test -z "$(git status --porcelain --untracked-files=all)"
printf '%s\n' "Candidate local checks passed at $reviewed_head; full-service pentest and GitHub acceptance remain required."
