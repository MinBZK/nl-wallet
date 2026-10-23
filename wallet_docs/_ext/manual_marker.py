"""Grammar of the `% manual` markers on the logical test cases page.

A marker declares that a logical test case cannot be covered by automated tests
and has to be executed by hand. It can be scoped to the platforms on which the
manual execution is needed, and can carry a short note for the tester:

    % manual                        manual on every platform
    % manual: ios                   automated on Android, manual on iOS
    % manual: android               automated on iOS, manual on Android
    % manual: ios - <reason>        with a note for the tester

Several markers per test case are allowed, which is how a per-platform note is
written:

    % manual: ios - Face ID cannot be simulated on real devices
    % manual: android - the fingerprint wizard differs per vendor

This module is the single definition of that grammar. It is used by the
`ltc_linker` Sphinx extension to render the markers on the documentation page,
and by `deploy/qa-util/extract-manual-test-set.py` to generate the manual test
set.
"""

import re
from dataclasses import dataclass

PLATFORM_LABELS = {
    'android': 'Android',
    'ios': 'iOS',
}


SYNTAX = '% manual[: <platforms>][ - <reason>]'

_PREFIX_RE = re.compile(r'^manual\b')

_MARKER_RE = re.compile(
    r'^manual\b'
    r'(?:\s*:\s*(?P<platforms>[a-z][a-z, ]+[a-z]))?'
    r'(?:\s+[-–—]\s+(?P<reason>.+))?$'
)


class MarkerError(ValueError):
    """Raised when a line looks like a manual marker but cannot be parsed."""


@dataclass(frozen=True)
class ManualMarker:
    """A single `% manual` marker."""

    platforms: tuple[str, ...]
    reason: str = ''

    @property
    def covers_all_platforms(self) -> bool:
        return all((p in self.platforms) for p in PLATFORM_LABELS)

    @property
    def platform_label(self) -> str:
        return ', '.join(PLATFORM_LABELS[platform] for platform in self.platforms)

    @property
    def scope_label(self) -> str:
        """Human readable scope, e.g. 'every platform' or 'iOS'."""
        if self.covers_all_platforms:
            return 'every platform'
        return self.platform_label

    @property
    def label(self) -> str:
        """Human readable marker, e.g. 'Manual test' or 'Manual test (iOS)'."""
        if self.covers_all_platforms:
            return 'Manual test'
        return f'Manual test ({self.platform_label})'


def parse_marker_body(body: str) -> ManualMarker | None:
    """Parse the body of a `%` comment (without the leading `%`).

    Returns None when the comment is not a manual marker, and raises
    MarkerError when it is one but malformed.
    """
    body = body.strip()
    if not _PREFIX_RE.match(body):
        return None

    match = _MARKER_RE.match(body)
    if match is None:
        raise MarkerError(f"cannot parse marker '% {body}', expected '{SYNTAX}'")

    raw_platforms = match.group('platforms')
    if raw_platforms is None:
        platforms = list(PLATFORM_LABELS)
    else:
        platforms = [name for name in re.split(r'[,\s]+', raw_platforms.strip()) if name]
        unknown = [name for name in platforms if name not in PLATFORM_LABELS]
        if unknown:
            raise MarkerError(
                f"marker '% {body}' has unknown platform(s) {', '.join(unknown)}, "
                f"expected one of {', '.join(PLATFORM_LABELS)}"
            )

    reason = (match.group('reason') or '').strip()

    return ManualMarker(platforms=tuple(sorted(set(platforms))), reason=reason)


def parse_marker_line(line: str) -> ManualMarker | None:
    """Parse a raw source line, which may or may not be a `%` comment."""
    if not line.startswith('%'):
        return None
    return parse_marker_body(line[1:].strip())


def merged_platforms(markers: list[ManualMarker]) -> tuple[str, ...]:
    """The union of the platforms of all markers of a single test case."""
    return tuple(sorted({platform for marker in markers for platform in marker.platforms}))


def covers_all_platforms(platforms: tuple[str, ...]) -> bool:
    platforms = frozenset(platforms)
    return all((p in platforms) for p in PLATFORM_LABELS)
