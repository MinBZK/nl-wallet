#!/usr/bin/env python3

import pathlib, re, sys
from collections import defaultdict
from typing import Dict, Set

DOC_PATH = pathlib.Path("wallet_docs/functional-design/logical-test-cases.md")
TEST_DIRS = [
    "uiautomation/src/test/kotlin",
    "wallet_core/tests_integration/tests",
    "browsertests/packages",
    "wallet_app/test",
]
EXTS = {".kt", ".kts", ".js", ".dart", ".rs"}

LTC_HEADER = re.compile(r"^###\s+(LTC\d+)")
LTC_TOKEN = re.compile(r"LTC\d+", re.IGNORECASE)


def check_prerequisites(doc_path: pathlib.Path, test_dirs: list[str]) -> None:
    print(f"Checking prerequisites...")

    # Check if the documentation file exists
    if not doc_path.is_file():
        print(f"Error: Required documentation file not found:")
        print(f"  Expected path: {doc_path.resolve()}")
        sys.exit(1)

    # Check if all test directories exist
    missing_dirs = []
    for directory in test_dirs:
        path = pathlib.Path(directory)
        if not path.is_dir():
            missing_dirs.append(directory)

    if missing_dirs:
        print("Error: The following required test directories were not found:")
        for directory in missing_dirs:
            print(f"  - {pathlib.Path(directory).resolve()}")
        sys.exit(1)

    print("Prerequisites OK.")


def load_ltcs(md_path: pathlib.Path) -> Set[str]:
    text = md_path.read_text()
    ids = []
    seen = set()
    manual_next = False
    for line in text.splitlines():
        if "<!--" in line and "Manual" in line:
            manual_next = True
            continue
        m = LTC_HEADER.match(line.strip())
        if m:
            ltc = m.group(1)  # doc is uppercase
            if manual_next:
                manual_next = False
                continue
            if ltc in seen:
                ids.append(ltc)
            seen.add(ltc)
            ids.append(ltc)
    dupes = sorted({x for x in ids if ids.count(x) > 1})
    if dupes:
        print(f"Duplicate LTC IDs in doc: {', '.join(dupes)}")
        sys.exit(1)
    return set(ids)


def scan_tests() -> Set[str]:
    found: Set[str] = set()
    for root in TEST_DIRS:
        for path in pathlib.Path(root).rglob("*"):
            if path.is_file() and path.suffix.lower() in EXTS:
                content = path.read_text(errors="ignore")
                for match in LTC_TOKEN.finditer(content):
                    token = match.group(0).upper()
                    found.add(token)
    return found


def main():
    doc_path = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else DOC_PATH
    check_prerequisites(doc_path, TEST_DIRS)
    required = load_ltcs(doc_path)
    found = scan_tests()

    missing = sorted(required - set(found))
    if missing:
        print("Missing LTCs (in doc, not in tests): " + ", ".join(missing))
        sys.exit(1)

    print("All required LTCs are referenced; no duplicates.")


if __name__ == "__main__":
    main()
