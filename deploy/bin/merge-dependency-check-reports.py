#!/usr/bin/env python3
"""Merge dependency-check reports into one JSON, XML and HTML report.

Dependencies with the same SHA-1 become one dependency, with the project
references and vulnerabilities of all of them.
"""

import argparse
import json
from html import escape
from pathlib import Path
from xml.etree import ElementTree

REPORT = "dependency-check-report"

# Lists to combine for equal dependencies, with the identity of an item
JSON_LISTS = {
    "projectReferences": lambda item: item,
    "includedBy": lambda item: item["reference"],
    "vulnerabilities": lambda item: item["name"],
    "suppressedVulnerabilities": lambda item: item["name"],
}
XML_LISTS = ("projectReferences", "includedBy", "vulnerabilities")


def deduplicate(dependencies, sha1, combine):
    """Return the dependencies with one dependency per SHA-1."""
    result = []
    first = {}
    for dependency in dependencies:
        key = sha1(dependency)
        if key in first:
            combine(first[key], dependency)
        else:
            result.append(dependency)
            # Virtual dependencies, such as from pubspec.lock, have no SHA-1
            if key:
                first[key] = dependency
    return result


def combine_json(first, other):
    for name, identity in JSON_LISTS.items():
        items = first.get(name, [])
        known = {identity(item) for item in items}
        items += [item for item in other.get(name, []) if identity(item) not in known]
        if items:
            first[name] = items


def merge_json(sources):
    reports = [json.loads(source.read_text()) for source in sources]
    report = reports[0]
    dependencies = [dependency for source in reports for dependency in source["dependencies"]]
    report["dependencies"] = deduplicate(dependencies, lambda dependency: dependency.get("sha1"), combine_json)
    return report


def xml_identity(item):
    # A vulnerability has a name, a project reference has only text
    return item.findtext("{*}name", item.text)


def combine_xml(first, other):
    for name in XML_LISTS:
        items = other.find(f"{{*}}{name}")
        if items is None:
            continue
        target = first.find(f"{{*}}{name}")
        if target is None:
            first.append(items)
            continue
        known = {xml_identity(item) for item in target}
        target.extend(item for item in items if xml_identity(item) not in known)


def merge_xml(sources):
    tree = ElementTree.parse(sources[0])
    root = tree.getroot()
    container = root.find("{*}dependencies")
    if container is None:
        raise SystemExit(f"{sources[0]}: no dependencies element")
    dependencies = list(container)
    for source in sources[1:]:
        source_root = ElementTree.parse(source).getroot()
        # The root tag contains the namespace of the schema version
        if source_root.tag != root.tag:
            raise SystemExit(f"{source}: schema {source_root.tag} is not {root.tag}")
        dependencies.extend(source_root.iterfind("{*}dependencies/{*}dependency"))
    container[:] = deduplicate(dependencies, lambda dependency: dependency.findtext("{*}sha1"), combine_xml)
    ElementTree.register_namespace("", root.tag[1:].split("}")[0])
    return tree


def html_row(dependency):
    packages = "<br>".join(escape(package["id"]) for package in dependency.get("packages", []))
    # Only the Gradle scans have project references
    users = "<br>".join(map(escape, dependency.get("projectReferences", []))) or escape(dependency["filePath"])
    vulnerabilities = "<br>".join(
        f'<a href="https://nvd.nist.gov/vuln/detail/{escape(item["name"])}">{escape(item["name"])}</a> '
        f"{escape(item.get('severity', ''))}"
        for item in dependency.get("vulnerabilities", [])
    )
    name = escape(dependency["fileName"])
    return f"<tr><td>{name}</td><td>{packages}</td><td>{users}</td><td>{vulnerabilities}</td></tr>"


def render_html(report):
    # Vulnerable dependencies first
    dependencies = sorted(
        report["dependencies"],
        key=lambda dependency: (not dependency.get("vulnerabilities"), dependency["fileName"].lower()),
    )
    vulnerable = sum(1 for dependency in dependencies if dependency.get("vulnerabilities"))
    rows = "\n".join(map(html_row, dependencies))
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Dependency-Check report</title>
<style>
body {{ font-family: sans-serif; font-size: small; }}
table {{ border-collapse: collapse; }}
th, td {{ border: 1px solid #ccc; padding: 4px; text-align: left; vertical-align: top; }}
</style>
</head>
<body>
<h1>Dependency-Check report</h1>
<p>{len(dependencies)} dependencies, {vulnerable} with vulnerabilities.</p>
<table>
<tr><th>Dependency</th><th>Packages</th><th>Used by</th><th>Vulnerabilities</th></tr>
{rows}
</table>
</body>
</html>
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("target", type=Path, help="directory to write the merged report to")
    parser.add_argument("sources", type=Path, nargs="+", help="directories with a JSON and XML report")
    args = parser.parse_args()
    report = merge_json([source / f"{REPORT}.json" for source in args.sources])
    (args.target / f"{REPORT}.json").write_text(json.dumps(report))
    (args.target / f"{REPORT}.html").write_text(render_html(report))
    tree = merge_xml([source / f"{REPORT}.xml" for source in args.sources])
    tree.write(args.target / f"{REPORT}.xml", encoding="UTF-8", xml_declaration=True)


if __name__ == "__main__":
    main()
