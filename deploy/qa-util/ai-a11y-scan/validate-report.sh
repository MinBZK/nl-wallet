#!/usr/bin/env bash
set -uo pipefail

REPORT="${1:-A11Y-REPORT.json}"
CAPTURES="${2:-captures}"
META="${3:-${REPORT%.json}.meta.json}"

fail() { echo "FAIL: $*" >&2; exit 1; }

[ -f "$REPORT" ] || fail "$REPORT not found (scan did not complete)"
[ -s "$REPORT" ] || fail "$REPORT is empty (scan did not complete)"

jq empty "$REPORT" 2>/dev/null \
  || fail "$REPORT is not valid JSON (write was interrupted)"

jq -e '
  (type == "array")
  and all(.[];
        (.url           | type == "string" and length > 0)
    and (.violations    | type == "array")
    and (.incomplete    | type == "array")
    and (.passes        | type == "array")
    and (.inapplicable  | type == "array")
    and (.testEngine    | type == "object")
    and (.testRunner    | type == "object")
    and (.testEnvironment | type == "object")
    and (.toolOptions   | type == "object")
    and (.timestamp     | type == "string" and length > 0)
    and all((.violations + .incomplete)[];
          (.id    | type == "string" and length > 0)
      and (.nodes | type == "array" and length > 0)))
' "$REPORT" >/dev/null || fail "$REPORT is not a valid axe array (see the schema in .claude/skills/a11y-scan/SKILL.md)"

screens=$(jq 'length' "$REPORT")
[ "$screens" -gt 0 ] || fail "$REPORT is an empty array (nothing was scanned)"

# Coverage: the scan emits exactly one result object per page-source capture, so
# a short array means screens were dropped rather than reviewed clean.
if [ -d "$CAPTURES" ]; then
  expected=$(find "$CAPTURES" -maxdepth 1 -name '*.xml' ! -name '._*' | wc -l | tr -d ' ')
  if [ "$expected" -gt 0 ] && [ "$screens" -ne "$expected" ]; then
    echo "Scanned screens:" >&2
    jq -r '.[].url' "$REPORT" | sort >&2
    fail "incomplete scan: $screens of $expected captures in $REPORT"
  fi
fi

# The sidecar holds everything the axe array has no slot for.
[ -f "$META" ] || fail "$META not found (scan did not write the metadata sidecar)"
jq empty "$META" 2>/dev/null \
  || fail "$META is not valid JSON (write was interrupted)"

meta_bad=$(jq -r '
  [ (if (.wcag_version|type)      != "string" then "wcag_version (want string, got \(.wcag_version|type))" else empty end),
    (if (.generated_at|type)      != "string" then "generated_at (want string, got \(.generated_at|type))" else empty end),
    (if (.suppressed|type)        != "array"  then "suppressed (want array, got \(.suppressed|type))" else empty end),
    (if (.summary|type)           != "object" then "summary (want object, got \(.summary|type))" else empty end),
    (if (.summary.screens|type)   != "number" then "summary.screens (want number, got \(.summary.screens|type))" else empty end),
    (if (.summary.violations|type)!= "number" then "summary.violations (want number, got \(.summary.violations|type))" else empty end)
  ] | join("; ")
' "$META")
[ -z "$meta_bad" ] || fail "$META is missing or mistyped: $meta_bad (see the schema in .claude/skills/a11y-scan/SKILL.md)"

# Provenance: report which commit each platform's captures came from, so the log
# says what was actually reviewed. Informational only — the captures are valid
# whatever commit produced them, so nothing here fails the scan.
if [ -f "$CAPTURES/sources.json" ]; then
  for platform in android ios; do
    jq -e --arg p "$platform" 'has($p)' "$CAPTURES/sources.json" >/dev/null 2>&1 || continue
    want=$(jq -r --arg p "$platform" '.[$p].commit' "$CAPTURES/sources.json")
    got=$(jq -r --arg p "$platform" '.capture_sources[$p].commit // "not recorded"' "$META")
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
reported=$(jq '.summary.screens' "$META")
[ "$reported" -eq "$screens" ] \
  || fail "summary.screens ($reported) disagrees with the axe array length ($screens)"

counted=$(jq '[.[].violations[]] | length' "$REPORT")
tallied=$(jq '.summary.violations' "$META")
[ "$counted" -eq "$tallied" ] \
  || fail "summary.violations ($tallied) disagrees with the findings ($counted)"

incomplete=$(jq '[.[].incomplete[]] | length' "$REPORT")
tallied_inc=$(jq '.summary.incomplete // "absent"' "$META")
[ "$tallied_inc" = "$incomplete" ] \
  || fail "summary.incomplete ($tallied_inc) disagrees with the findings ($incomplete)"

echo "$REPORT OK: $screens screens, $counted violations, $incomplete incomplete, $(jq '.suppressed | length' "$META") suppressed"
