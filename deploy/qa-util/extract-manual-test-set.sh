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

/^###[[:space:]]+LTC[0-9]+/ {
  buffer = $0 "\n"
  is_manual = 0
  in_section = 1
  next
}

in_section && /^%[[:space:]]*manual[[:space:]]*$/ {
  is_manual = 1
  next
}

in_section && /^---[[:space:]]*$/ {
  if (is_manual) {
    printf "%s", buffer
    print "---\n"
  }
  buffer = ""
  is_manual = 0
  in_section = 0
  next
}

in_section {
  buffer = buffer $0 "\n"
}
' "$in" > "$out"
