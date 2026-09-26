"""Non-executing, fail-closed discovery of the reviewed Scaleway docs layout."""

from html.parser import HTMLParser
import re

from scaleway_source_fetch import InventoryError

ORIGIN = "https://www.scaleway.com"
INDEX = ORIGIN + "/en/developers/api"
VERSION = r"v[1-9][0-9]*(?:(?:alpha|beta)[1-9][0-9]*)?"
SPEC = re.compile(rf"openapi/openapi/scaleway\.([a-z0-9_]+)\.({VERSION})\.([A-Za-z0-9]+)\.yml")
STRING = r"`[^`\\${}\x00-\x1f]*`"
PAIR = rf"[A-Za-z][A-Za-z0-9]*:{STRING}"
OBJECT = rf"\{{{PAIR}(?:,{PAIR})*\}}"


class IndexParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.scripts: list[str] = []
        self.routes: set[str] = set()

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if len(attributes) != len(attrs):
            raise InventoryError("duplicate index HTML attribute")
        if tag == "script" and attributes.get("type") == "module":
            source = attributes.get("src", "")
            if not isinstance(source, str) or not re.fullmatch(
                r"/en/developers/assets/entry\.client-[A-Za-z0-9_-]+\.js", source
            ):
                raise InventoryError("unreviewed index module script")
            self.scripts.append(ORIGIN + source)
        if tag == "a":
            href = attributes.get("href") or ""
            prefix = "/en/developers/api/"
            if href.startswith(prefix):
                self.routes.add("/api/" + href.removeprefix(prefix).split("#")[0].rstrip("/"))


def index_sources(raw: bytes) -> tuple[str, set[str], str]:
    text = raw.decode("utf-8")
    parser = IndexParser()
    parser.feed(text)
    if len(parser.scripts) != 1 or not parser.routes:
        raise InventoryError("index script or navigation discovery incomplete")
    builds = set(re.findall(r"v1\.[0-9]+\.[0-9]+", text))
    if len(builds) != 1:
        raise InventoryError("ambiguous documentation build")
    return parser.scripts[0], parser.routes, builds.pop()


def discover(raw: bytes, navigation: set[str]) -> list[dict]:
    text = raw.decode("utf-8")
    # Only accept the flat literal catalog grammar; never evaluate JavaScript.
    starts = list(re.finditer(r"=\[\{category:`[^`]*`,label:`", text))
    if len(starts) != 1:
        raise InventoryError("missing or ambiguous docs catalog")
    start = starts[0].start() + 1
    match = re.match(rf"\[{OBJECT}(?:,{OBJECT})*\]", text[start:])
    if match is None:
        raise InventoryError("catalog literal grammar changed")
    records = []
    identities = set()
    catalog = match.group()
    for obj in re.findall(OBJECT, catalog):
        pairs = re.findall(rf"([A-Za-z][A-Za-z0-9]*):({STRING})", obj)
        entry = {key: value[1:-1] for key, value in pairs}
        if len(entry) != len(pairs) or set(entry) - {
            "category", "label", "group", "route", "specFile", "visibility", "href", "badge"
        }:
            raise InventoryError("duplicate or unknown catalog field")
        if not {"category", "label", "route", "specFile"} <= entry.keys():
            raise InventoryError("catalog entry missing required fields")
        spec = SPEC.fullmatch(entry["specFile"])
        if spec is None or not re.fullmatch(r"/api/[a-z0-9_/-]+", entry["route"]):
            raise InventoryError("malformed catalog version or route")
        if entry.get("visibility", "public") not in {"public", "unlisted"}:
            raise InventoryError("unknown catalog visibility")
        family, version, interface = spec.groups()
        identity = (entry["route"], version)
        if identity in identities:
            raise InventoryError("duplicate catalog route/version")
        identities.add(identity)
        entry.update(family=family, version=version, interface=interface)
        records.append(entry)
    if not records or len(records) > 256:
        raise InventoryError("catalog entry count outside bounds")
    routes = {row["route"].rstrip("/") for row in records}
    missing = navigation - routes - {"/api/quickstart"}
    if missing:
        raise InventoryError(f"navigation routes absent from catalog: {sorted(missing)}")
    # The independent file-input registry must agree on every schema version.
    inputs = re.findall(r"type:`file`,input:\[(.*?)\],path:`(/api/[^`]+)`", text)
    discovered = set()
    for values, route in inputs:
        versions = re.findall(rf"path:`({VERSION})`,label:`[^`]+`,input:`({SPEC.pattern})`", values)
        # This registry is only a discovery hint; a mismatch stops rather than
        # silently narrowing the accepted catalog when the bundle layout changes.
        for value in versions:
            version, file = value[:2]
            key = (route, version, file)
            if key in discovered:
                raise InventoryError("duplicate schema input registry entry")
            discovered.add(key)
    expected = {(r["route"], r["version"], r["specFile"]) for r in records if "href" not in r}
    if discovered != expected:
        raise InventoryError("schema registry and catalog disagree")
    return records
