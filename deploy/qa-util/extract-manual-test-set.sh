#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 input.md output.md" >&2
  exit 1
fi

in="$1"
out="$2"

if [[ ! -f $in ]]; then
  echo "Input file not found: $in" >&2
  exit 1
fi

awk '
BEGIN {
  print "# Manual Logical Test Cases\n"
}

/<!--[[:space:]]*Manual[[:space:]]*-->/ {
  pending = 1
  next
}

pending && /^###[[:space:]]+LTC[0-9]+/ {
  capture = 1
  pending = 0
}

capture {
  print
  if (/^---[[:space:]]*$/) {
    print ""
    capture = 0
  }
}
' "$in" > "$out"
