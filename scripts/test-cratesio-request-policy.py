#!/usr/bin/env python3
"""Offline mutation tests for the crates.io request-policy source gate."""

import json
import shutil
import tempfile
from pathlib import Path

from check_cratesio_request_policy import CRATE, ROOT, PolicyError, validate
from cratesio_source_error import SourceLockError


def rejected(root, online=False, fetcher=None):
    try:
        if fetcher is None:
            validate(root, online)
        else:
            validate(root, online, fetcher)
    except (PolicyError, SourceLockError):
        return
    raise AssertionError("mutated policy was accepted")


def main():
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        files = [CRATE / "identifiers/mod.rs", CRATE / "query/mod.rs",
            CRATE / "query/contract_tests.rs", Path("provider-drift/providers/cratesio-source.lock.json")]
        for path in files:
            (root / path).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, root / path)
        validate(root)
        changes = [
            (files[0], "MAX_VERSION_BYTES: usize = 150", "MAX_VERSION_BYTES: usize = 151"),
            (files[1], "MAX_PER_PAGE: u32 = 100", "MAX_PER_PAGE: u32 = 101"),
            (files[1], "MAX_PAGE: u32 = 10", "MAX_PAGE: u32 = 11"),
            (files[2], "QueryOperation::Categories", "QueryOperation::Unknown"),
        ]
        for path, old, new in changes:
            text = (root / path).read_text()
            assert old in text
            (root / path).write_text(text.replace(old, new, 1))
            rejected(root)
            (root / path).write_text(text)
        path = root / files[3]
        text = path.read_text()
        data = json.loads(text)
        data["source_commit"] = "0" * 40
        path.write_text(json.dumps(data))
        rejected(root)
        path.write_text(text)

        def unavailable(_):
            raise SourceLockError("offline fetch rejection")
        rejected(root, True, unavailable)
        rejected(root, True, lambda _: b'{"paths":{}}')
        validate(root)
    print("8 crates.io request-policy regression groups passed.")


if __name__ == "__main__":
    main()
