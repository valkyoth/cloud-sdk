#!/usr/bin/env python3
"""Archive inspector rejects malformed members without extracting them."""

import io
from pathlib import Path
import tarfile
import tempfile

from check_cratesio_archives import inspect


def main():
    prefix = "cloud-sdk-cratesio-1.1.0"
    required = [f"{prefix}/{name}" for name in
                ("Cargo.toml", "Cargo.lock", "README.md", "src/lib.rs")]
    with tempfile.TemporaryDirectory() as directory:
        for extra in (None, f"{prefix}/../escape", f"{prefix}/.env",
                      f"{prefix}/token.key", required[0], "/absolute", "symlink"):
            path = Path(directory) / "test.crate"
            with tarfile.open(path, "w:gz") as archive:
                for name in required + ([] if extra is None else [extra]):
                    member = tarfile.TarInfo(name)
                    if name == "symlink":
                        member.type = tarfile.SYMTYPE
                        member.linkname = "/outside"
                        archive.addfile(member)
                    else:
                        member.size = 1
                        archive.addfile(member, io.BytesIO(b"x"))
            try:
                inspect(path, prefix)
            except ValueError:
                assert extra is not None
            else:
                assert extra is None, extra
    print("7 archive member validation cases passed.")


if __name__ == "__main__":
    main()
