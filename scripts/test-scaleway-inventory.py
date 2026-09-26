#!/usr/bin/env python3
"""Offline adversarial tests for Scaleway discovery, parsing, and retrieval."""

import copy
from decimal import Decimal
import gzip
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from check_scaleway_inventory import DIRECTORY, Sources, build_parser, observe, qualify, unresolved
from scaleway_discovery import INDEX, bounded_discovery, discover, index_sources, registry_inputs
from scaleway_inventory import check_references, load_json, operations, parse_schema, sdk_candidates
from scaleway_source_fetch import InventoryError, MAX_SOURCE, MAX_TOTAL, NoRedirects, SDK_TREE, approved_url, fetch, read_bounded

CATALOG = b'var x=[{category:`Compute`,label:`Instance`,route:`/api/instance`,specFile:`openapi/openapi/scaleway.instance.v1.Api.yml`}];'
REGISTRY = b'var y=[{type:`file`,input:[{path:`v1`,label:`v1`,input:`openapi/openapi/scaleway.instance.v1.Api.yml`}],path:`/api/instance`}];'
SCHEMA = b'''openapi: 3.1.0
info: {version: v1, title: Example}
paths:
  /example:
    get: {operationId: GetExample, responses: {'200': {description: success}}}
'''


class DiscoveryTests(unittest.TestCase):
    def test_unterminated_registry_is_rejected_within_subprocess_budget(self):
        probe = '''
import sys
from scaleway_discovery import parse_catalog
from scaleway_source_fetch import InventoryError
try:
    parse_catalog(sys.stdin.buffer.read(), {"/api/instance"})
except InventoryError:
    sys.exit(0)
sys.exit(1)
'''
        payload = CATALOG + b"type:`file`,input:[" * 100_000
        result = subprocess.run([sys.executable, "-c", probe], input=payload,
                                cwd=Path(__file__).parent, timeout=5, check=False)
        self.assertEqual(result.returncode, 0)

    def test_registry_scanner_bounds_and_nested_markers(self):
        for value in [b"type:`file`,input:[", REGISTRY[:-5],
                      REGISTRY.replace(b"{path:", b"type:`file`,input:[{path:"),
                      REGISTRY.replace(b"/api/instance", b"//attacker.invalid"),
                      REGISTRY.replace(b"{path:", b"x" * (1024 * 1024) + b"{path:")]:
            with self.subTest(length=len(value)), self.assertRaises(InventoryError):
                list(registry_inputs(value.decode()))

    def test_discovery_stalled_worker_is_terminated(self):
        original = subprocess.run
        def stalled(command, **kwargs):
            return original([sys.executable, "-c", "import time; time.sleep(60)"], **kwargs)
        for kind in ["index", "catalog"]:
            with self.subTest(kind=kind), patch("scaleway_discovery.subprocess.run", side_effect=stalled):
                with self.assertRaisesRegex(InventoryError, "deadline"):
                    bounded_discovery(kind, b"", set(), timeout=0.05)

    def test_discovery_input_and_output_limits(self):
        with self.assertRaises(InventoryError):
            discover(b"x" * (MAX_SOURCE + 1), set())
        with self.assertRaises(InventoryError):
            discover(b"", {str(i) for i in range(257)})
        with patch("scaleway_discovery.subprocess.run") as run:
            run.return_value.stdout = b"x" * (MAX_SOURCE * 6 + 1)
            with self.assertRaises(InventoryError):
                discover(b"", set())

    def test_catalog_and_independent_registry(self):
        rows = discover(CATALOG + REGISTRY, {"/api/instance"})
        self.assertEqual(rows[0]["version"], "v1")

    def test_catalog_mutations_rejected(self):
        for raw in [CATALOG, REGISTRY, CATALOG + CATALOG + REGISTRY,
                    (CATALOG + REGISTRY).replace(b"v1", b"v01"),
                    (CATALOG + REGISTRY).replace(b"category:`Compute`", b"category:`${run()}`"),
                    (CATALOG + REGISTRY).replace(b"category:`Compute`", b"category:`Compute`,category:`Other`"),
                    (CATALOG + REGISTRY).replace(b"category:`Compute`", b"unknown:`Compute`"),
                    CATALOG + REGISTRY.replace(b"v1", b"v2"),
                    (CATALOG + REGISTRY).replace(b"route:`/api/instance`", b"route:`/api/instance`,visibility:`hidden`")]:
            with self.subTest(raw=raw), self.assertRaises(InventoryError):
                discover(raw, {"/api/instance"})

    def test_navigation_cannot_be_silently_dropped(self):
        with self.assertRaises(InventoryError):
            discover(CATALOG + REGISTRY, {"/api/new-product"})

    def test_index_requires_unique_module_build_and_navigation(self):
        page = b'<script type="module" src="/en/developers/assets/entry.client-test.js"></script><a href="/en/developers/api/instance">v1.100.0</a>'
        self.assertEqual(index_sources(page)[2], "v1.100.0")
        for value in [b"", page + page, page.replace(b"test.js", b"../../test.js"),
                      page.replace(b"src=", b'src="a" src='), page + b"v1.101.0"]:
            with self.subTest(value=value), self.assertRaises(InventoryError):
                index_sources(value)


class SchemaTests(unittest.TestCase):
    def test_numeric_values_survive_the_complete_bridge(self):
        for number in ["9223372036854775808", "-9223372036854775809",
                       "18446744073709551615", "0.12345678901234567890123456789",
                       "1e400", "1e-400", "-0.0", "1.00E+02"]:
            with self.subTest(number=number):
                schema = parse_schema(SCHEMA + f"constraint: {number}\n".encode())
                self.assertEqual(schema["constraint"], Decimal(number))
                if any(char in number for char in ".eE"):
                    self.assertIsInstance(schema["constraint"], Decimal)
                    self.assertEqual(schema["constraint"].as_tuple(), Decimal(number).as_tuple())
        for number in ["+1", ".5", "1.", "01", "0x10", "0o10", ".nan", ".inf"]:
            with self.subTest(number=number), self.assertRaises(InventoryError):
                parse_schema(SCHEMA + f"constraint: {number}\n".encode())

    def test_operation_paths_use_canonical_template_policy(self):
        schema = parse_schema(SCHEMA)
        entry = {"family": "instance", "version": "v1"}
        item = schema["paths"]["/example"]
        for path in ["/", "/example/{project_id}", "/v1/{zone}/thing:action",
                     "/encoded%20space", "/" + "a" * 8191]:
            with self.subTest(path=path):
                schema["paths"] = {path: item}
                self.assertEqual(operations(entry, schema)[0]["path"], path)
        for path in ["", None, 7, "//attacker.invalid/x", "/ok?admin=true", "/../admin",
                     "/path\\segment", "/contains space", "/a//b", "/a/./b", "/a#b",
                     "/a\x00b", "/a\x7fb", "/cafe\u00e9", "/%", "/%0", "/%GG",
                     "/%2F", "/%5C", "/%3F", "/%23", "/%25", "/%00", "/%7F",
                     "/%2E%2E/a", "/%41", "/%3a", "/{name", "/name}", "/{}",
                     "/{bad-name}", "/{{name}}", "/" + "a" * 8192]:
            with self.subTest(path=path), self.assertRaises(InventoryError):
                schema["paths"] = {path: item}
                operations(entry, schema)

    def test_valid_schema_and_operations(self):
        schema = parse_schema(SCHEMA)
        rows = operations({"family": "instance", "version": "v1"}, schema)
        self.assertEqual(rows[0]["method"], "GET")
        self.assertFalse(rows[0]["deprecated"])

    def test_yaml_rejection(self):
        for suffix in [b"info: {}\n", b"a: &a [1]\n", b"a: *a\n", b"a: !!str hi\n",
                       b"a: !bad hi\n", b"a: {x: 1, x: 2}\n", b"<<: {}\n",
                       b"a: .inf\n", b"---\na: 1\n", b"a: [\n"]:
            with self.subTest(suffix=suffix), self.assertRaises(InventoryError):
                parse_schema(SCHEMA + suffix)
        with self.assertRaises(InventoryError):
            parse_schema(b"a: " + b"[" * 65 + b"0" + b"]" * 65)

    def test_reference_policy(self):
        check_references({"x": {"$ref": "#/x"}})
        check_references({"a/b": {}, "x": {"$ref": "#/a~1b"}})
        for ref in ["https://attacker.invalid/a", "file:///tmp/a", "other.yml#/a",
                    "#/missing", "#/a~2b", "#", None]:
            with self.subTest(ref=ref), self.assertRaises(InventoryError):
                check_references({"x": {"$ref": ref}})

    def test_invalid_operation_metadata(self):
        schema = parse_schema(SCHEMA)
        for mutation in ["version", "method", "id", "deprecated", "duplicate"]:
            value = copy.deepcopy(schema)
            if mutation == "version":
                value["info"]["version"] = "v2"
            elif mutation == "method":
                value["paths"]["/example"]["CONNECT"] = {}
            elif mutation == "id":
                del value["paths"]["/example"]["get"]["operationId"]
            elif mutation == "deprecated":
                value["paths"]["/example"]["get"]["deprecated"] = "false"
            else:
                value["paths"]["/other"] = copy.deepcopy(value["paths"]["/example"])
            with self.subTest(mutation=mutation), self.assertRaises(InventoryError):
                operations({"family": "instance", "version": "v1"}, value)

    def test_sdk_tree_is_complete(self):
        tree = {"truncated": False, "tree": [{"path": "api/billing/v2/billing_sdk.go"}]}
        self.assertEqual(sdk_candidates(tree, [])[0]["disposition"], "unresolved")
        self.assertEqual(sdk_candidates(tree, [{"family": "billing", "version": "v2"}]), [])
        tree["truncated"] = True
        with self.assertRaises(InventoryError):
            sdk_candidates(tree, [])

    def test_duplicate_json_rejected(self):
        for raw in [b'{"a":1,"a":2}', b'{"a":NaN}']:
            with self.assertRaises(InventoryError):
                load_json(raw)


class FetchTests(unittest.TestCase):
    def test_url_admission(self):
        approved_url(INDEX)
        approved_url(SDK_TREE)
        for url in ["http://www.scaleway.com/en/developers/api", INDEX + "?token=a",
                    INDEX + "#fragment", INDEX.replace("www.", "user@www."),
                    INDEX.replace("www.", "@www."), INDEX.replace(".com", ".com:0"),
                    INDEX.replace("www.scaleway.com", "localhost"),
                    INDEX.replace(".com", ".com:443"), INDEX + "/../secret",
                    INDEX.replace("/api", "/assets/%2e%2e/secret.js"),
                    "https://api.scaleway.com/account/v3/projects", INDEX + "\n"]:
            with self.subTest(url=url), self.assertRaises(InventoryError):
                approved_url(url)

    def test_redirects_rejected(self):
        with self.assertRaises(InventoryError):
            NoRedirects().redirect_request(None, None, 302, "redirect", {}, INDEX)

    def test_byte_and_encoding_limits(self):
        response = io.BytesIO(b"abcd")
        response.headers = {}
        self.assertEqual(read_bounded(response, 4), b"abcd")
        for headers in [{}, {"Content-Length": "-1"}, {"Content-Length": "9"},
                        {"Content-Length": "invalid"}, {"Content-Encoding": "gzip"}]:
            response = io.BytesIO(b"abcde")
            response.headers = headers
            with self.subTest(headers=headers), self.assertRaises(InventoryError):
                read_bounded(response, 4)

    def test_real_stalled_worker_is_terminated(self):
        original = subprocess.run
        def stalled(command, **kwargs):
            return original([sys.executable, "-c", "import time; time.sleep(60)"], **kwargs)
        started = time.monotonic()
        with patch("scaleway_source_fetch.subprocess.run", side_effect=stalled):
            with self.assertRaisesRegex(InventoryError, "deadline"):
                fetch(INDEX, timeout=0.05)
        self.assertLess(time.monotonic() - started, 5)

    def test_source_budget_archive_and_digest(self):
        with tempfile.TemporaryDirectory() as tmp:
            sources = Sources(Path(tmp))
            sources.total = MAX_TOTAL
            with self.assertRaises(InventoryError):
                sources.get(INDEX)
            sources.total = 0
            sources.started -= 1801
            with self.assertRaises(InventoryError):
                sources.get(INDEX)
            sources.started = time.monotonic()
            with self.assertRaises(InventoryError):
                sources.get(INDEX, {"sha256": "../outside"})
            import hashlib
            raw = b"test"
            digest = hashlib.sha256(raw).hexdigest()
            path = Path(tmp) / "sources" / (digest + ".gz")
            path.parent.mkdir()
            path.write_bytes(gzip.compress(raw, mtime=0))
            expected = {"url": INDEX, "sha256": digest, "size_bytes": 4}
            self.assertEqual(sources.get(INDEX, expected), raw)
            with self.assertRaises(InventoryError):
                sources.get(INDEX, expected)
            expected["size_bytes"] = 5
            with self.assertRaises(InventoryError):
                Sources(Path(tmp)).get(INDEX, expected)

    def test_worker_output_and_limits_are_checked(self):
        with patch("scaleway_source_fetch.subprocess.run") as run:
            run.return_value.stdout = b"abcde"
            with self.assertRaises(InventoryError):
                fetch(INDEX, 4)
            for limit in [0, -1, True, MAX_SOURCE + 1]:
                with self.subTest(limit=limit), self.assertRaises(InventoryError):
                    fetch(INDEX, limit)

    def test_source_count_and_expansion_are_bounded(self):
        with tempfile.TemporaryDirectory() as tmp:
            sources = Sources(Path(tmp))
            sources.records = [{"url": str(i)} for i in range(256)]
            with self.assertRaises(InventoryError):
                sources.get(INDEX)
            sources.records = []
            sources.total = MAX_TOTAL - 4
            digest = "0" * 64
            target = Path(tmp) / "sources" / (digest + ".gz")
            target.parent.mkdir()
            target.write_bytes(gzip.compress(b"x" * 10000, mtime=0))
            with self.assertRaises(InventoryError):
                sources.get(INDEX, {"sha256": digest})


class SnapshotTests(unittest.TestCase):
    def test_committed_snapshot_reproduces(self):
        inventory = load_json((DIRECTORY / "inventory.json").read_bytes())
        rebuilt = observe(Sources(DIRECTORY), inventory["sources"])
        rebuilt["observed_at"] = inventory["observed_at"]
        self.assertEqual(rebuilt, inventory)

    def test_qualification_does_not_hide_unlisted_candidates(self):
        inventory = load_json((DIRECTORY / "inventory.json").read_bytes())
        self.assertIn("SDK billing v2", unresolved(inventory))
        self.assertIn("/api/account/organization v3", unresolved(inventory))

    def test_source_records_cannot_be_removed_reordered_or_appended(self):
        inventory = load_json((DIRECTORY / "inventory.json").read_bytes())
        records = inventory["sources"]
        for altered in [records[1:], records[::-1], records + [records[0]]]:
            with self.subTest(length=len(altered)), self.assertRaises(InventoryError):
                observe(Sources(DIRECTORY), altered)

    def test_followup_approval_is_exact_and_never_authorizes_release(self):
        raw = (DIRECTORY / "inventory.json").read_bytes()
        inventory = load_json(raw)
        approved = load_json((DIRECTORY / "followups.json").read_bytes())
        qualify(inventory, raw, approved)
        with self.assertRaisesRegex(InventoryError, "release NOT qualified"):
            qualify(inventory, raw, approved, release=True)
        for field, replacement in [("inventory_sha256", "0" * 64), ("candidates", []),
                                   ("resolve_by_checkpoint", 80),
                                   ("release_allowed_with_unresolved", True),
                                   ("authorization", "")]:
            altered = copy.deepcopy(approved)
            altered[field] = replacement
            with self.subTest(field=field), self.assertRaises(InventoryError):
                qualify(inventory, raw, altered)
        altered_inventory = copy.deepcopy(inventory)
        altered_inventory["sdk_only_candidates"].append({
            "family": "new", "version": "v1", "disposition": "unresolved"})
        with self.assertRaises(InventoryError):
            qualify(altered_inventory, raw, approved)


if __name__ == "__main__":
    build_parser()
    unittest.main()
