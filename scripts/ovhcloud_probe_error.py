#!/usr/bin/env python3
"""Shared OVHcloud probe failure type."""


class OvhcloudProbeError(RuntimeError):
    """Official probe sources do not match the reviewed shape."""
