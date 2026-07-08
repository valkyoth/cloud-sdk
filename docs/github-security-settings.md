# GitHub Security Settings

Enable these repository settings:

- Dependabot version updates for Cargo and GitHub Actions.
- Dependabot alerts.
- Secret scanning.
- Private vulnerability reporting.
- CodeQL default setup.
- Branch protection for `main`.
- Required pull request review before merging to `main`.
- Required status check for the Rust CI job before merging to `main`.
- Optional signed-commit enforcement if the repository policy allows it.

Do not add an advanced CodeQL workflow while default setup is active.
