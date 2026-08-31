#!/usr/bin/env python3
"""Shared crates.io source-lock error type."""


class SourceLockError(ValueError):
    """The upstream source or committed lock is incomplete."""
