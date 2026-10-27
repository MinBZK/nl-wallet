#!/usr/bin/env python3
"""Generate the manual test set from the logical test cases page.

Every logical test case carrying a '% manual' marker is copied into the output,
grouped by the platforms on which it has to be executed by hand. See
wallet_docs/_ext/manual_marker.py for the marker grammar.
"""

import argparse
import re
import sys
from dataclasses import dataclass
from dataclasses import field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / 'wallet_docs' / '_ext'))

from manual_marker import (  # noqa: E402
    PLATFORM_LABELS,
    ManualMarker,
    MarkerError,
    covers_all_platforms,
    merged_platforms,
    parse_marker_line,
)

_SECTION_RE = re.compile(r'^###\s+LTC(?P<id>\d+)\b')
_TITLE_RE = re.compile(r'^####\s+(?P<title>.+?)\s*$')
_END_RE = re.compile(r'^---\s*$')


@dataclass
class TestCase:
    """A single '### LTCn' section of the logical test cases page."""

    ltc_id: str
    heading: str
    line_number: int
    body: list[str] = field(default_factory=list)
    markers: list[ManualMarker] = field(default_factory=list)

    @property
    def title(self) -> str:
        for line in self.body:
            match = _TITLE_RE.match(line)
            if match:
                return match.group('title')
        return f'LTC{self.ltc_id}'

    @property
    def platforms(self) -> tuple[str, ...]:
        return merged_platforms(self.markers)


def parse(path: Path) -> tuple[list[TestCase], list[str]]:
    """Parse the page into test cases, and return those plus any marker errors."""
    cases: list[TestCase] = []
    errors: list[str] = []
    current: TestCase | None = None

    for line_number, line in enumerate(path.read_text(encoding='utf-8').splitlines(), start=1):
        section = _SECTION_RE.match(line)
        if section:
            current = TestCase(ltc_id=section.group('id'), heading=line.rstrip(), line_number=line_number)
            cases.append(current)
            continue

        if current is None:
            continue

        if _END_RE.match(line):
            current = None
            continue

        try:
            marker = parse_marker_line(line)
        except MarkerError as e:
            errors.append(f'{path}:{line_number}: LTC{current.ltc_id}: {e}')
            continue

        if marker is not None:
            current.markers.append(marker)
        else:
            current.body.append(line.rstrip())

    return cases, errors


def note_lines(case: TestCase) -> list[str]:
    """Render the markers of a test case as a note for the tester."""
    lines: list[str] = []

    for marker in case.markers:
        if marker.covers_all_platforms and not marker.reason:
            continue
        note = f'> **Manual on {marker.scope_label}**'
        if marker.reason:
            note += f' — {marker.reason}'
        lines.append(note)

    return lines


def squeeze_blanks(lines: list[str]) -> list[str]:
    """Collapse the blank runs left behind by the removed markers, and strip the outer blanks.

    The tail has to be blank free so that render_case fully controls the separator it appends.
    """
    squeezed: list[str] = []

    for line in lines:
        if not line.strip() and (not squeezed or not squeezed[-1].strip()):
            continue
        squeezed.append(line)

    while squeezed and not squeezed[-1].strip():
        squeezed.pop()

    return squeezed


def render_case(case: TestCase) -> list[str]:
    """Render a single test case: its original content plus the marker notes."""
    body = squeeze_blanks(case.body)

    notes = note_lines(case)
    if notes:
        insert_at = 0
        for index, line in enumerate(body):
            if _TITLE_RE.match(line):
                insert_at = index + 1
                break
        body[insert_at:insert_at] = ['', *notes]

    return [case.heading, '', *body, '', '---', '']


def scope_sections(cases: list[TestCase]) -> list[tuple[str, list[TestCase]]]:
    """Group the test cases by the platforms they are manual on."""
    sections: list[tuple[str, list[TestCase]]] = [
        ('Manual on every platform', [c for c in cases if covers_all_platforms(c.platforms)]),
    ]

    for platform, label in PLATFORM_LABELS.items():
        sections.append((
            f'Manual on {label} only',
            [c for c in cases if c.platforms == (platform,)],
        ))

    return sections


def render(cases: list[TestCase], source: Path, total: int) -> str:
    """Render the complete manual test set."""
    sections = scope_sections(cases)

    lines = [
        '# Manual Logical Test Cases',
        '',
        f'Generated from `{source}` by `deploy/qa-util/extract-manual-test-set.py`.',
        'Do not edit by hand.',
        '',
        f'{len(cases)} of {total} logical test cases need manual execution: ' +
        ', '.join(f'{len(section_cases)} on {heading[len("Manual on "):]}' for heading, section_cases in sections) +
        '.',
        '',
    ]

    if cases:
        lines += [
            '| LTC | Test case | Manual on |',
            '| --- | --- | --- |',
        ]
        for case in cases:
            scope = 'every platform' if covers_all_platforms(case.platforms) else ', '.join(
                PLATFORM_LABELS[platform] for platform in case.platforms
            )
            lines.append(f'| [LTC{case.ltc_id}](#ltc{case.ltc_id}) | {case.title} | {scope} |')
        lines.append('')

    for heading, section_cases in sections:
        if not section_cases:
            continue
        lines += [f'## {heading}', '']
        for case in section_cases:
            lines += render_case(case)

    return '\n'.join(lines).rstrip() + '\n'


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path, help='the logical test cases page')
    parser.add_argument('output', type=Path, help='the manual test set to write')
    args = parser.parse_args()

    if not args.input.is_file():
        print(f'Input file not found: {args.input}', file=sys.stderr)
        return 1

    cases, errors = parse(args.input)

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    manual_cases = [case for case in cases if case.markers]
    args.output.write_text(render(manual_cases, args.input, len(cases)), encoding='utf-8')

    print(f'Wrote {len(manual_cases)} of {len(cases)} logical test cases to {args.output}')

    return 0


if __name__ == '__main__':
    sys.exit(main())
