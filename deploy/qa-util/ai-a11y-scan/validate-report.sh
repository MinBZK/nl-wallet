#!/usr/bin/env bash
set -uo pipefail

REPORT="${1:-A11Y-REPORT.json}"
CAPTURES="${2:-captures}"

fail() { echo "FAIL: $*" >&2; exit 1; }

[ -f "$REPORT" ] || fail "$REPORT not found (scan did not complete)"
[ -s "$REPORT" ] || fail "$REPORT is empty (scan did not complete)"

jq empty "$REPORT" 2>/dev/null \
  || fail "$REPORT is not valid JSON (write was interrupted)"

jq -e '
  (.wcag_version | type == "string")
  and (.generated_at | type == "string")
  and (.results     | type == "array")
  and (.suppressed  | type == "array")
  and (.summary     | type == "object")
  and (.summary.screens    | type == "number")
  and (.summary.violations | type == "number")
  and all(.results[];
        (.url        | type == "string" and length > 0)
    and (.violations | type == "array")
    and (.incomplete | type == "array")
    and all((.violations + .incomplete)[];
          (.id    | type == "string" and length > 0)
      and (.nodes | type == "array" and length > 0)))
' "$REPORT" >/dev/null || fail "$REPORT is structurally invalid (see the schema in .claude/skills/a11y-scan/SKILL.md)"

screens=$(jq '.results | length' "$REPORT")
[ "$screens" -gt 0 ] || fail "$REPORT has an empty results array (nothing was scanned)"

# Coverage: the scan emits exactly one result object per page-source capture, so
# a short results array means screens were dropped rather than reviewed clean.
if [ -d "$CAPTURES" ]; then
  expected=$(find "$CAPTURES" -maxdepth 1 -name '*.xml' ! -name '._*' | wc -l | tr -d ' ')
  if [ "$expected" -gt 0 ] && [ "$screens" -ne "$expected" ]; then
    echo "Scanned screens:" >&2
    jq -r '.results[].url' "$REPORT" | sort >&2
    fail "incomplete scan: $screens of $expected captures in $REPORT"
  fi
fi

# Provenance: report which commit each platform's captures came from, so the log
# says what was actually reviewed. Informational only — the captures are valid
# whatever commit produced them, so nothing here fails the scan.
if [ -f "$CAPTURES/sources.json" ]; then
  for platform in android ios; do
    jq -e --arg p "$platform" 'has($p)' "$CAPTURES/sources.json" >/dev/null 2>&1 || continue
    want=$(jq -r --arg p "$platform" '.[$p].commit' "$CAPTURES/sources.json")
    got=$(jq -r --arg p "$platform" '.capture_sources[$p].commit // "not recorded"' "$REPORT")
    if [ "$got" = "$want" ]; then
      echo "$platform captures: $want"
    else
      echo "$platform captures: $want (report says '$got')"
    fi
  done
  echo "scan commit: $(jq -r '.scan_commit // "unknown"' "$CAPTURES/sources.json")"
fi

# Internal consistency: a summary that disagrees with the findings means the
# report was assembled from a partial set.
reported=$(jq '.summary.screens' "$REPORT")
[ "$reported" -eq "$screens" ] \
  || fail "summary.screens ($reported) disagrees with results length ($screens)"

counted=$(jq '[.results[].violations[]] | length' "$REPORT")
tallied=$(jq '.summary.violations' "$REPORT")
[ "$counted" -eq "$tallied" ] \
  || fail "summary.violations ($tallied) disagrees with the findings ($counted)"

echo "$REPORT OK: $screens screens, $counted violations, $(jq '.summary.incomplete' "$REPORT") incomplete, $(jq '.suppressed | length' "$REPORT") suppressed"
