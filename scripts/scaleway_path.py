"""Canonical runtime path policy, extended only for OpenAPI placeholders."""

import re
import string

from scaleway_source_fetch import InventoryError

UNRESERVED = frozenset(string.ascii_letters + string.digits + "-._~")
PATH_BYTES = UNRESERVED | frozenset("/!$&'()*+,;=:@")
HEX = frozenset("0123456789ABCDEF")
MAX_PATH = 8192  # cloud_sdk::transport::MAX_REQUEST_TARGET_BYTES


def validate_operation_path(path: object) -> str:
    if not isinstance(path, str) or not 0 < len(path) <= MAX_PATH or not path.isascii():
        raise InventoryError("invalid operation path")
    # Validate templates before substituting a harmless runtime segment.
    canonical = re.sub(r"\{[A-Za-z_][A-Za-z0-9_]*\}", "parameter", path)
    if not canonical.startswith("/") or "//" in canonical:
        raise InventoryError("non-canonical operation path")
    if any(segment in {".", ".."} for segment in canonical.split("/")):
        raise InventoryError("operation path contains dot segment")
    cursor = 0
    while cursor < len(canonical):
        byte = canonical[cursor]
        if byte == "%":
            encoded = canonical[cursor + 1:cursor + 3]
            if len(encoded) != 2 or any(digit not in HEX for digit in encoded):
                raise InventoryError("non-canonical percent escape")
            decoded = chr(int(encoded, 16))
            if decoded in UNRESERVED or decoded in "/\\?%#" or ord(decoded) < 32 or ord(decoded) == 127:
                raise InventoryError("unsafe encoded operation path")
            cursor += 3
        elif byte in PATH_BYTES:
            cursor += 1
        else:
            raise InventoryError("unsafe operation path byte or placeholder")
    return path
