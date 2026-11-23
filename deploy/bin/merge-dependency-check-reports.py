#!/usr/bin/env python3
"""Merge dependency-check reports into one JSON, XML and HTML report.

Usage: merge-dependency-check-reports.py TARGET SOURCE...

TARGET is the directory to write the merged report to. Each SOURCE is a
directory with a JSON and XML report. Dependencies with the same SHA-1
become one dependency, with the project references and vulnerabilities
of all of them. Vulnerable dependencies come first, so the HTML anchors
match the Quality-time links of both dependencies and security warnings.
The HTML report also shows the files and packages of the related
dependencies that dependency-check bundled into a dependency.

A SOURCE without a report, for example because its scan failed, is
skipped with a warning.
"""

import json
import sys
from collections.abc import Callable, Iterable
from html import escape
from pathlib import Path
from string import Template
from typing import NotRequired, TypedDict, cast
from xml.etree import ElementTree

REPORT = "dependency-check-report"
XML_LISTS = ("projectReferences", "includedBy", "vulnerabilities")
HTML_VULN = Template('<a href="https://nvd.nist.gov/vuln/detail/$name">$name</a> $severity')
HTML_ROW = Template('<tr id="l${index}_${sha1}"><td>$name</td><td>$packages</td><td>$users</td><td>$vulns</td></tr>')
HTML_OUT = Template("""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Dependency-Check report</title>
<style>
body { font-family: sans-serif; font-size: small; }
table { border-collapse: collapse; }
th, td { border: 1px solid #ccc; padding: 4px; text-align: left; vertical-align: top; }
</style>
</head>
<body>
<h1>Dependency-Check report</h1>
<p>$total dependencies, $vulnerable with vulnerabilities.</p>
<table>
<tr><th>Dependency</th><th>Packages</th><th>Used by</th><th>Vulnerabilities</th></tr>
$rows
</table>
</body>
</html>
""")


class Vulnerability(TypedDict):
    name: str
    severity: NotRequired[str]


class Package(TypedDict):
    id: str


class Reference(TypedDict):
    reference: str


class RelatedDependency(TypedDict):
    filePath: str
    packageIds: NotRequired[list[Package]]


class Dependency(TypedDict):
    fileName: str
    filePath: str
    sha1: NotRequired[str]
    packages: NotRequired[list[Package]]
    relatedDependencies: NotRequired[list[RelatedDependency]]
    projectReferences: NotRequired[list[str]]
    includedBy: NotRequired[list[Reference]]
    vulnerabilities: NotRequired[list[Vulnerability]]
    suppressedVulnerabilities: NotRequired[list[Vulnerability]]


class Report(TypedDict):
    dependencies: list[Dependency]


def deduplicate[T](items: Iterable[T], sha1: Callable[[T], str | None], combine: Callable[[T, T], None]) -> list[T]:
    first: dict[object, T] = {}
    for item in items:
        kept = first.setdefault(sha1(item) or id(item), item)
        if kept is not item:
            combine(kept, item)
    return list(first.values())


def union[T](items: list[T], others: list[T], key: Callable[[T], str]) -> list[T]:
    known = {key(item) for item in items}
    return items + [item for item in others if key(item) not in known]


def combine_json(first: Dependency, other: Dependency) -> None:
    first["projectReferences"] = union(first.get("projectReferences", []), other.get("projectReferences", []), str)
    first["includedBy"] = union(
        first.get("includedBy", []), other.get("includedBy", []), lambda item: item["reference"]
    )
    for name in ("vulnerabilities", "suppressedVulnerabilities"):
        first[name] = union(first.get(name, []), other.get(name, []), lambda item: item["name"])


def merge_json(sources: list[Path]) -> Report:
    reports = [cast(Report, json.loads(source.read_text())) for source in sources]
    dependencies = deduplicate(
        (dependency for report in reports for dependency in report["dependencies"]),
        lambda dependency: dependency.get("sha1"),
        combine_json,
    )
    report = reports[0]
    report["dependencies"] = sorted(dependencies, key=lambda dependency: not dependency.get("vulnerabilities"))
    return report


def xml_identity(item: ElementTree.Element) -> str | None:
    return item.findtext("{*}name", item.text)


def combine_xml(first: ElementTree.Element, other: ElementTree.Element) -> None:
    for name in XML_LISTS:
        items = other.find(f"{{*}}{name}")
        target = first.find(f"{{*}}{name}")
        if items is None:
            continue
        if target is None:
            first.append(items)
        else:
            known = {xml_identity(item) for item in target}
            target.extend(item for item in items if xml_identity(item) not in known)


def merge_xml(sources: list[Path]) -> ElementTree.Element:
    roots = [ElementTree.parse(source).getroot() for source in sources]
    for source, root in zip(sources, roots):
        if root.tag != roots[0].tag:
            raise SystemExit(f"{source}: schema {root.tag} is not {roots[0].tag}")
    container = roots[0].find("{*}dependencies")
    if container is None:
        raise SystemExit(f"{sources[0]}: no dependencies element")
    dependencies = deduplicate(
        (dependency for root in roots for dependency in root.iterfind("{*}dependencies/{*}dependency")),
        lambda dependency: dependency.findtext("{*}sha1"),
        combine_xml,
    )
    container[:] = sorted(
        dependencies, key=lambda dependency: dependency.find("{*}vulnerabilities/{*}vulnerability") is None
    )
    ElementTree.register_namespace("", roots[0].tag[1:].split("}")[0])
    return roots[0]


def html_vuln(vuln: Vulnerability) -> str:
    return HTML_VULN.substitute(name=escape(vuln["name"]), severity=escape(vuln.get("severity", "")))


def html_row(index: int, dependency: Dependency) -> str:
    related = dependency.get("relatedDependencies", [])
    packages = [package["id"] for package in dependency.get("packages", [])]
    packages += [package["id"] for item in related for package in item.get("packageIds", [])]
    # Show the projects of Gradle dependencies, else all scanned files
    users = dependency.get("projectReferences") or [dependency["filePath"], *(item["filePath"] for item in related)]
    return HTML_ROW.substitute(
        index=index,
        sha1=escape(dependency.get("sha1", "")),
        name=escape(dependency["fileName"]),
        packages="<br>".join(map(escape, dict.fromkeys(packages))),
        users="<br>".join(map(escape, users)),
        vulns="<br>".join(map(html_vuln, dependency.get("vulnerabilities", []))),
    )


def html_out(report: Report) -> str:
    dependencies = report["dependencies"]
    vulnerable = sum(1 for dependency in dependencies if dependency.get("vulnerabilities"))
    rows = "\n".join(html_row(index, dependency) for index, dependency in enumerate(dependencies, 1))
    return HTML_OUT.substitute(total=len(dependencies), vulnerable=vulnerable, rows=rows)


def main() -> None:
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    target, *sources = map(Path, sys.argv[1:])
    missing = [source for source in sources if not (source / f"{REPORT}.json").is_file()]
    for source in missing:
        print(f"WARNING: {source}: no report, skipping", file=sys.stderr)
    sources = [source for source in sources if source not in missing]
    if not sources:
        raise SystemExit("ERROR: no reports to merge")
    report = merge_json([source / f"{REPORT}.json" for source in sources])
    _ = (target / f"{REPORT}.json").write_text(json.dumps(report))
    _ = (target / f"{REPORT}.html").write_text(html_out(report))
    root = merge_xml([source / f"{REPORT}.xml" for source in sources])
    ElementTree.ElementTree(root).write(target / f"{REPORT}.xml", encoding="UTF-8", xml_declaration=True)


if __name__ == "__main__":
    main()
