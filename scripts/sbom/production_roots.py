#!/usr/bin/env python3
"""Select production SBOM roots from cargo metadata."""

import argparse
import json


def production_roots(metadata):
    members = set(metadata["workspace_members"])
    for package in metadata["packages"]:
        if package["id"] not in members:
            continue

        settings = (package.get("metadata") or {}).get("sbom", {})
        is_root = settings.get("production-root")
        if is_root is not None and not isinstance(is_root, bool):
            raise ValueError(f"{package['name']}: package.metadata.sbom.production-root must be a boolean")
        if is_root is None:
            is_root = any(
                kind in {"bin", "staticlib", "cdylib"}
                for target in package["targets"]
                for kind in target["kind"]
            )
        if is_root:
            yield package["manifest_path"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metadata", type=argparse.FileType("r", encoding="utf-8"))
    args = parser.parse_args()
    try:
        roots = list(production_roots(json.load(args.metadata)))
        if not roots:
            raise ValueError("No production Rust roots found")
    except (OSError, ValueError) as error:
        parser.exit(1, f"{parser.prog}: {error}\n")
    print("\n".join(roots))


if __name__ == "__main__":
    main()
