#!/usr/bin/env python3
"""Normalize CycloneDX references and merge duplicate component observations."""

import argparse
import json
import sys


class NormalizationError(ValueError):
    pass


def json_sort_key(value):
    """Compare JSON values consistently, including objects with different key order."""
    if value is None:
        return (0,)
    if isinstance(value, bool):
        return (1, value)
    if isinstance(value, (int, float)):
        return (2, value)
    if isinstance(value, str):
        return (3, value)
    if isinstance(value, list):
        return (4, tuple(json_sort_key(item) for item in value))
    keys = tuple(sorted(value))
    return (5, keys, tuple(json_sort_key(value[key]) for key in keys))


def unique_values(values):
    """Deduplicate array metadata without depending on dictionary insertion order."""
    by_value = {json_sort_key(value): value for value in values}
    return [by_value[key] for key in sorted(by_value)]


def merge_by_reference(items, reference_key, keep_unreferenced=False):
    groups = {}
    for item in items:
        groups.setdefault(item.get(reference_key), []).append(item)

    merged = []
    for reference in sorted(groups, key=json_sort_key):
        group = groups[reference]
        if reference is None and keep_unreferenced:
            merged.extend(group)
        else:
            merged.append(merge_observations(group))
    return merged


def merge_observations(observations):
    """Combine metadata, rejecting scalar conflicts instead of dropping information."""
    merged = {}
    for observation in observations:
        for key, right in observation.items():
            left = merged.get(key)
            if left is None:
                merged[key] = right
            elif right is None or json_sort_key(left) == json_sort_key(right):
                continue
            elif key == "scope":
                # A package required by any root remains required in the inventory.
                priorities = {"required": 0, "optional": 1, "excluded": 2}
                merged[key] = min((left, right), key=lambda scope: priorities.get(scope, -1))
            elif isinstance(left, dict) and isinstance(right, dict):
                merged[key] = merge_observations([left, right])
            elif isinstance(left, list) and isinstance(right, list):
                if key == "components":
                    merged[key] = merge_by_reference(left + right, "bom-ref", keep_unreferenced=True)
                else:
                    merged[key] = unique_values(left + right)
            else:
                raise NormalizationError(f"Conflicting component field: {key}")
    return merged


def iter_objects(value):
    if isinstance(value, dict):
        yield value
        for child in value.values():
            yield from iter_objects(child)
    elif isinstance(value, list):
        for child in value:
            yield from iter_objects(child)


def normalize_objects(value, references):
    """Rewrite references and coalesce component lists, processing children first."""
    if isinstance(value, list):
        return [normalize_objects(child, references) for child in value]
    if not isinstance(value, dict):
        return value

    result = {key: normalize_objects(child, references) for key, child in value.items()}
    for key in ("bom-ref", "ref"):
        if key in result:
            result[key] = references.get(result[key], result[key])
    if "dependsOn" in result:
        result["dependsOn"] = unique_values(references.get(ref, ref) for ref in result["dependsOn"])
    if "components" in result:
        result["components"] = merge_by_reference(result["components"], "bom-ref", keep_unreferenced=True)
    return result


def normalize(bom):
    if not isinstance(bom, dict):
        raise NormalizationError("Expected a CycloneDX JSON object")

    # References are local to an input BOM. Package URLs identify components
    # across inputs, while retaining distinct versions and URL qualifiers.
    references = {}
    for component in iter_objects(bom):
        reference = component.get("bom-ref")
        purl = component.get("purl")
        if reference is not None and purl is not None:
            if reference in references and references[reference] != purl:
                raise NormalizationError(f"Ambiguous component reference: {reference}")
            references[reference] = purl

    result = normalize_objects(bom, references)

    # cdxgen can repeat the project in the top-level component list. Keep its
    # combined metadata under metadata.component, including nested targets.
    root = (result.get("metadata") or {}).get("component") or {}
    root_reference = root.get("bom-ref")
    if root_reference is not None and "components" in result:
        duplicates = [item for item in result["components"] if item.get("bom-ref") == root_reference]
        result["metadata"]["component"] = merge_observations([root] + duplicates)
        result["components"] = [item for item in result["components"] if item.get("bom-ref") != root_reference]

    # Missing dependency information must stay missing, rather than becoming
    # an empty graph that claims the component has no dependencies.
    if "dependencies" in result:
        result["dependencies"] = merge_by_reference(result["dependencies"], "ref")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", nargs="?", type=argparse.FileType("r", encoding="utf-8"), default=sys.stdin)
    args = parser.parse_args()
    try:
        result = normalize(json.load(args.input))
    except (OSError, ValueError) as error:
        parser.exit(1, f"{parser.prog}: {error}\n")
    json.dump(result, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
