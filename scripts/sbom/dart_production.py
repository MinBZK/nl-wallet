#!/usr/bin/env python3
"""Build a production Dart SBOM from cdxgen components and Pub's resolved graph."""

import argparse
import copy
import json
import sys
from uuid import uuid4

from normalize import normalize


def production_bom(bom, graph, root):
    packages = {}
    for package in graph["packages"]:
        name = package["name"]
        if name in packages:
            raise ValueError(f"Duplicate Pub package: {name}")
        packages[name] = package

    # In a Pub workspace every member has kind=root, and dependencies includes
    # its dev dependencies. Follow directDependencies from the application only.
    reachable = set()
    pending = [root]
    while pending:
        name = pending.pop()
        if name in reachable:
            continue
        if name not in packages:
            raise ValueError(f"Missing runtime package in Pub graph: {name}")
        dependencies = packages[name].get("directDependencies")
        if not isinstance(dependencies, list) or any(not isinstance(dep, str) for dep in dependencies):
            raise ValueError(f"{name}: expected a directDependencies list from pub deps --json")
        reachable.add(name)
        pending.extend(dependencies)

    result = normalize(copy.deepcopy(bom))
    inventory = {
        (component["name"], component.get("version")): component
        for component in [result.get("metadata", {}).get("component", {}), *result.get("components", [])]
        if component.get("purl", "").startswith("pkg:pub/")
    }
    components = {}
    for name in sorted(reachable):
        package = packages[name]
        version = package["version"]
        component = inventory.get((name, version))
        if component is None:
            raise ValueError(f"Missing SBOM component for runtime package: {name}@{version}")
        component["type"] = "application" if name == root else "library"
        component["scope"] = "required"
        components[name] = component

    result["serialNumber"] = f"urn:uuid:{uuid4()}"
    result.setdefault("metadata", {})["component"] = components[root]
    result["components"] = [component for name, component in components.items() if name != root]
    # Rebuild edges from Pub: cdxgen's lockfile graph cannot distinguish
    # development-only transitive dependencies in a workspace.
    result["dependencies"] = [
        {
            "ref": component["bom-ref"],
            "dependsOn": sorted({components[dep]["bom-ref"] for dep in packages[name]["directDependencies"]}),
        }
        for name, component in components.items()
    ]
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bom", type=argparse.FileType("r", encoding="utf-8"))
    parser.add_argument("dependencies", type=argparse.FileType("r", encoding="utf-8"))
    parser.add_argument("--root", required=True, help="Production application package name")
    args = parser.parse_args()
    try:
        result = production_bom(json.load(args.bom), json.load(args.dependencies), args.root)
    except (KeyError, TypeError, OSError, ValueError) as error:
        parser.exit(1, f"{parser.prog}: {error}\n")
    json.dump(result, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
