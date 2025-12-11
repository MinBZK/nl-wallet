#!/bin/bash

# Get the directory of this script.
scripts_dir="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd -P)"

# Output claim properties in tsv format, extends-aware.
function json_claims_to_tsv() {
    local file="$1"
    local parent="$2"

    local header='(["Claim Path","Label","Description","Language"],'
    local filter='| (.path|join(".")) as $path 
                  | .display[] 
                  | [$path, .label, .description, .lang])) 
                  | @tsv'

    if [[ $parent ]]; then
        claims='(($parent[0].claims + .claims)[] '
        jq -r --slurpfile parent "$parent" "$header$claims$filter" "$file"
    else
        claims='(.claims[] '
        jq -r "$header$claims$filter" "$file"
    fi
}

# Output a given tsv as markdown table.
function tsv_to_markdown_table() {
    local tsv="$1"
    echo "$tsv" | awk 'BEGIN{FS=OFS="\t"} {n=(NF>n?NF:n); for(i=1;i<=NF;i++) {if(length($i)>w[i]) w[i]=length($i)} rows[NR]=$0} END{for(r=1;r<=NR;r++){split(rows[r],f,FS); printf("|"); for(i=1;i<=n;i++){printf(" %-*s |", w[i], f[i])} printf("\n"); if(r==1){printf("|"); for(i=1;i<=n;i++){dash=""; for(j=1;j<=w[i];j++) dash=dash"-"; printf(" %s |", dash)} printf("\n")}}}'
}

# Iterate over VCT json documents, print header, generate tables.
for file in $scripts_dir/devenv/eudi:*.json; do
    extends=$(jq -r '.extends // empty' "$file")

    printf "\n## Claims in $(basename ${file/.json/})\n\n"

    if [ -n "$extends" ]; then
        tsv_to_markdown_table "$(json_claims_to_tsv "$file" "$scripts_dir/devenv/${extends#urn:}.json")"
    else
        tsv_to_markdown_table "$(json_claims_to_tsv "$file")"
    fi
done
