#!/usr/bin/env python3
"""Add the dependencies of dependency-check reports to a target report."""

import argparse
import json
import xml.etree.ElementTree as ElementTree


def merge_json(target, sources):
    with open(target) as file:
        report = json.load(file)
    for source in sources:
        with open(source) as file:
            report["dependencies"].extend(json.load(file)["dependencies"])
    with open(target, "w") as file:
        json.dump(report, file)


def merge_xml(target, sources):
    tree = ElementTree.parse(target)
    root = tree.getroot()
    # The root tag contains the namespace of the schema version
    namespace = root.tag[1:].split("}")[0]
    dependencies = root.find(f"{{{namespace}}}dependencies")
    if dependencies is None:
        raise SystemExit(f"{target}: no dependencies element")
    for source in sources:
        source_root = ElementTree.parse(source).getroot()
        if source_root.tag != root.tag:
            raise SystemExit(f"{source}: schema {source_root.tag} is not {root.tag}")
        dependencies.extend(source_root.iterfind(f"{{{namespace}}}dependencies/{{{namespace}}}dependency"))
    ElementTree.register_namespace("", namespace)
    tree.write(target, encoding="UTF-8", xml_declaration=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", help="report to add to, .json or .xml")
    parser.add_argument("sources", nargs="*", help="reports with the same format")
    args = parser.parse_args()
    merge = merge_xml if args.target.endswith(".xml") else merge_json
    merge(args.target, args.sources)


if __name__ == "__main__":
    main()
